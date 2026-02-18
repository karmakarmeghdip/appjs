// Custom Module Loader for AppJS
//
// Supports:
// - Local files (.js, .ts, .jsx, .tsx, .mjs, .mts, .json)
// - https:// URLs (remote ES modules)
// - jsr: specifiers (resolved via https://jsr.io)
// - npm: specifiers (resolved via https://esm.sh)
//
// TypeScript/JSX/TSX files are transpiled to JavaScript using deno_ast.
// Source maps are stored for better error reporting.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::pin::Pin;
use std::rc::Rc;

use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceMapOption;
use deno_core::ModuleLoadOptions;
use deno_core::ModuleLoadReferrer;
use deno_core::ModuleLoadResponse;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceCode;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::ResolutionKind;
use deno_core::error::ModuleLoaderError;
use deno_core::resolve_import;
use deno_core::serde_json::Value;
use deno_error::JsErrorBox;

type SourceMapStore = Rc<RefCell<HashMap<String, Vec<u8>>>>;

pub struct AppJsModuleLoader {
    source_maps: SourceMapStore,
    http_client: reqwest::Client,
}

impl AppJsModuleLoader {
    pub fn new() -> Self {
        Self {
            source_maps: Rc::new(RefCell::new(HashMap::new())),
            http_client: reqwest::Client::new(),
        }
    }
}

/// Resolve a jsr: specifier to an https URL via the JSR registry.
/// Format: jsr:@scope/package[@version][/path]
fn resolve_jsr_specifier(specifier: &str) -> Result<ModuleSpecifier, JsErrorBox> {
    let rest = specifier
        .strip_prefix("jsr:")
        .ok_or_else(|| JsErrorBox::generic("Not a jsr: specifier"))?;

    // Parse @scope/package[@version][/path]
    let rest = rest.trim_start_matches('/');

    if !rest.starts_with('@') {
        return Err(JsErrorBox::generic(
            "jsr: specifier must start with @scope/package",
        ));
    }

    // Parse @scope/package[@version][/path...]
    let scope_end = rest
        .find('/')
        .ok_or_else(|| JsErrorBox::generic("jsr: specifier must be @scope/package"))?;

    let scope = &rest[..scope_end]; // @scope
    let after_scope = &rest[scope_end + 1..];

    if after_scope.is_empty() {
        return Err(JsErrorBox::generic("jsr: specifier must be @scope/package"));
    }

    let (package_and_version, subpath) = if let Some(path_sep) = after_scope.find('/') {
        (&after_scope[..path_sep], &after_scope[path_sep + 1..])
    } else {
        (after_scope, "")
    };

    let (package, version) = if let Some(at_pos) = package_and_version.rfind('@') {
        (
            &package_and_version[..at_pos],
            &package_and_version[at_pos + 1..],
        )
    } else {
        (package_and_version, "")
    };

    if package.is_empty() {
        return Err(JsErrorBox::generic("jsr: specifier missing package name"));
    }

    // Construct the jsr.io URL
    // https://jsr.io/@scope/package[@version][/path]
    let mut url_str = if version.is_empty() {
        format!("https://jsr.io/{scope}/{package}")
    } else {
        format!("https://jsr.io/{scope}/{package}@{version}")
    };

    if !subpath.is_empty() {
        url_str.push('/');
        url_str.push_str(subpath);
    };

    ModuleSpecifier::parse(&url_str)
        .map_err(|e| JsErrorBox::generic(format!("Failed to parse jsr URL '{}': {}", url_str, e)))
}

fn is_version_like(segment: &str) -> bool {
    let core = segment
        .split_once('-')
        .map(|(left, _)| left)
        .unwrap_or(segment);

    let mut parts = core.split('.');
    let (Some(major), Some(minor), Some(patch)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };

    if parts.next().is_some() {
        return false;
    }

    major.chars().all(|c| c.is_ascii_digit())
        && minor.chars().all(|c| c.is_ascii_digit())
        && patch.chars().all(|c| c.is_ascii_digit())
}

fn parse_jsr_path(path: &str) -> Option<(&str, &str, Option<&str>, Option<String>)> {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if segments.len() < 2 {
        return None;
    }

    let scope = segments[0];
    if !scope.starts_with('@') {
        return None;
    }

    let pkg_segment = segments[1];
    let (package, version) = if let Some(at_pos) = pkg_segment.rfind('@') {
        (&pkg_segment[..at_pos], Some(&pkg_segment[at_pos + 1..]))
    } else {
        (pkg_segment, None)
    };

    if package.is_empty() {
        return None;
    }

    let subpath = if segments.len() > 2 {
        Some(segments[2..].join("/"))
    } else {
        None
    };

    Some((scope, package, version, subpath))
}

fn is_jsr_package_entry_url(specifier: &ModuleSpecifier) -> bool {
    if specifier.scheme() != "https" {
        return false;
    }

    if specifier.host_str() != Some("jsr.io") {
        return false;
    }

    let segments: Vec<&str> = specifier
        .path()
        .trim_start_matches('/')
        .split('/')
        .collect();
    if segments.len() < 2 {
        return false;
    }

    if !segments[0].starts_with('@') {
        return false;
    }

    if segments.len() >= 3 && is_version_like(segments[2]) {
        return false;
    }

    true
}

async fn resolve_jsr_entry_to_module(
    client: &reqwest::Client,
    specifier: &ModuleSpecifier,
) -> Result<ModuleSpecifier, JsErrorBox> {
    let (scope, package, requested_version, subpath) = parse_jsr_path(specifier.path())
        .ok_or_else(|| JsErrorBox::generic(format!("Invalid JSR entry URL: {}", specifier)))?;

    let package_meta_url = format!("https://jsr.io/{scope}/{package}/meta.json");
    let package_meta_text = client
        .get(&package_meta_url)
        .send()
        .await
        .map_err(|e| JsErrorBox::generic(format!("Failed to fetch '{}': {}", package_meta_url, e)))?
        .error_for_status()
        .map_err(|e| {
            JsErrorBox::generic(format!("HTTP error fetching '{}': {}", package_meta_url, e))
        })?
        .text()
        .await
        .map_err(|e| {
            JsErrorBox::generic(format!("Failed to read '{}': {}", package_meta_url, e))
        })?;
    let package_meta: Value = deno_core::serde_json::from_str(&package_meta_text).map_err(|e| {
        JsErrorBox::generic(format!("Invalid JSON from '{}': {}", package_meta_url, e))
    })?;

    let version = if let Some(v) = requested_version {
        v.to_string()
    } else {
        package_meta
            .get("latest")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                JsErrorBox::generic(format!("Missing latest version in '{}'", package_meta_url))
            })?
            .to_string()
    };

    let version_meta_url = format!("https://jsr.io/{scope}/{package}/{version}_meta.json");
    let version_meta_text = client
        .get(&version_meta_url)
        .send()
        .await
        .map_err(|e| JsErrorBox::generic(format!("Failed to fetch '{}': {}", version_meta_url, e)))?
        .error_for_status()
        .map_err(|e| {
            JsErrorBox::generic(format!("HTTP error fetching '{}': {}", version_meta_url, e))
        })?
        .text()
        .await
        .map_err(|e| {
            JsErrorBox::generic(format!("Failed to read '{}': {}", version_meta_url, e))
        })?;
    let version_meta: Value = deno_core::serde_json::from_str(&version_meta_text).map_err(|e| {
        JsErrorBox::generic(format!("Invalid JSON from '{}': {}", version_meta_url, e))
    })?;

    let exports = version_meta
        .get("exports")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            JsErrorBox::generic(format!("Missing exports map in '{}'", version_meta_url))
        })?;

    let export_key = if let Some(path) = subpath.as_ref().filter(|p| !p.is_empty()) {
        format!("./{path}")
    } else {
        ".".to_string()
    };

    let export_target = exports
        .get(&export_key)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            JsErrorBox::generic(format!(
                "Export key '{}' not found for '{}@{}'",
                export_key, scope, package
            ))
        })?;

    let relative = export_target
        .trim_start_matches("./")
        .trim_start_matches('/');
    let resolved_url = format!("https://jsr.io/{scope}/{package}/{version}/{relative}");

    ModuleSpecifier::parse(&resolved_url).map_err(|e| {
        JsErrorBox::generic(format!(
            "Failed to parse resolved JSR module URL '{}': {}",
            resolved_url, e
        ))
    })
}

/// Resolve an npm: specifier via esm.sh CDN.
/// Format: npm:package[@version][/path]
fn resolve_npm_specifier(specifier: &str) -> Result<ModuleSpecifier, JsErrorBox> {
    let rest = specifier
        .strip_prefix("npm:")
        .ok_or_else(|| JsErrorBox::generic("Not an npm: specifier"))?;

    // esm.sh serves npm packages as ES modules
    let url_str = format!("https://esm.sh/{rest}");
    ModuleSpecifier::parse(&url_str)
        .map_err(|e| JsErrorBox::generic(format!("Failed to parse npm URL '{}': {}", url_str, e)))
}

/// Determine MediaType from URL, using both path extension and Content-Type header.
fn media_type_from_specifier(specifier: &ModuleSpecifier) -> MediaType {
    let path = specifier.path();
    if let Some(ext) = path.rsplit('.').next() {
        match ext {
            "ts" | "mts" | "cts" => MediaType::TypeScript,
            "tsx" => MediaType::Tsx,
            "js" | "mjs" | "cjs" => MediaType::JavaScript,
            "jsx" => MediaType::Jsx,
            "json" => MediaType::Json,
            _ => {
                // For URLs without clear extension, default to TypeScript
                // (jsr/esm.sh often serve TS or modules without extension)
                if specifier.scheme() == "https" || specifier.scheme() == "http" {
                    MediaType::TypeScript
                } else {
                    MediaType::Unknown
                }
            }
        }
    } else if specifier.scheme() == "https" || specifier.scheme() == "http" {
        MediaType::TypeScript
    } else {
        MediaType::Unknown
    }
}

/// Determine MediaType from Content-Type header value.
fn media_type_from_content_type(content_type: &str, specifier: &ModuleSpecifier) -> MediaType {
    let ct = content_type.split(';').next().unwrap_or("").trim();
    match ct {
        "application/typescript" | "text/typescript" => MediaType::TypeScript,
        "application/javascript" | "text/javascript" | "application/x-javascript" => {
            MediaType::JavaScript
        }
        "application/json" | "text/json" => MediaType::Json,
        "text/tsx" => MediaType::Tsx,
        "text/jsx" => MediaType::Jsx,
        _ => media_type_from_specifier(specifier),
    }
}

/// Transpile TypeScript/JSX/TSX code to JavaScript using deno_ast.
fn transpile(
    specifier: &ModuleSpecifier,
    code: String,
    media_type: MediaType,
    source_maps: &SourceMapStore,
) -> Result<String, JsErrorBox> {
    let jsx_runtime = match media_type {
        MediaType::Jsx | MediaType::Tsx => {
            Some(deno_ast::JsxRuntime::Classic(deno_ast::JsxClassicOptions {
                factory: "jsx".to_string(),
                fragment_factory: "Fragment".to_string(),
            }))
        }
        _ => None,
    };

    let parsed = deno_ast::parse_module(ParseParams {
        specifier: specifier.clone(),
        text: code.into(),
        media_type,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })
    .map_err(JsErrorBox::from_err)?;

    let res = parsed
        .transpile(
            &deno_ast::TranspileOptions {
                imports_not_used_as_values: deno_ast::ImportsNotUsedAsValues::Remove,
                decorators: deno_ast::DecoratorsTranspileOption::Ecma,
                jsx: jsx_runtime,
                ..Default::default()
            },
            &deno_ast::TranspileModuleOptions { module_kind: None },
            &deno_ast::EmitOptions {
                source_map: SourceMapOption::Separate,
                inline_sources: true,
                ..Default::default()
            },
        )
        .map_err(JsErrorBox::from_err)?;

    let res = res.into_source();
    if let Some(source_map) = res.source_map {
        source_maps
            .borrow_mut()
            .insert(specifier.to_string(), source_map.into_bytes());
    }
    Ok(res.text)
}

/// Load a local file module (synchronous).
fn load_local(
    specifier: &ModuleSpecifier,
    source_maps: &SourceMapStore,
) -> Result<ModuleSource, ModuleLoaderError> {
    let path = specifier
        .to_file_path()
        .map_err(|_| JsErrorBox::generic("Failed to convert specifier to file path"))?;

    let media_type = MediaType::from_path(&path);
    let (module_type, should_transpile) = match media_type {
        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => (ModuleType::JavaScript, false),
        MediaType::Jsx => (ModuleType::JavaScript, true),
        MediaType::TypeScript
        | MediaType::Mts
        | MediaType::Cts
        | MediaType::Dts
        | MediaType::Dmts
        | MediaType::Dcts
        | MediaType::Tsx => (ModuleType::JavaScript, true),
        MediaType::Json => (ModuleType::Json, false),
        _ => {
            return Err(JsErrorBox::generic(format!(
                "Unknown extension {:?}",
                path.extension()
            )));
        }
    };

    let code = std::fs::read_to_string(&path).map_err(JsErrorBox::from_err)?;
    let code = if should_transpile {
        transpile(specifier, code, media_type, source_maps)?
    } else {
        code
    };

    Ok(ModuleSource::new(
        module_type,
        ModuleSourceCode::String(code.into()),
        specifier,
        None,
    ))
}

impl ModuleLoader for AppJsModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, ModuleLoaderError> {
        // Handle npm: specifiers
        if specifier.starts_with("npm:") {
            return resolve_npm_specifier(specifier);
        }

        // Handle jsr: specifiers
        if specifier.starts_with("jsr:") {
            return resolve_jsr_specifier(specifier);
        }

        // Handle https: and http: specifiers directly
        if specifier.starts_with("https://") || specifier.starts_with("http://") {
            return ModuleSpecifier::parse(specifier).map_err(|e| {
                ModuleLoaderError::from(JsErrorBox::generic(format!(
                    "Invalid URL '{}': {}",
                    specifier, e
                )))
            });
        }

        // For relative imports from an https module, resolve against the referrer
        if referrer.starts_with("https://") || referrer.starts_with("http://") {
            return resolve_import(specifier, referrer).map_err(JsErrorBox::from_err);
        }

        // Default: resolve as relative file path import
        resolve_import(specifier, referrer).map_err(JsErrorBox::from_err)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
        let scheme = module_specifier.scheme();

        match scheme {
            "file" => {
                // Synchronous local file load
                let source_maps = self.source_maps.clone();
                ModuleLoadResponse::Sync(load_local(module_specifier, &source_maps))
            }
            "https" | "http" | "jsr" => {
                // Async remote module fetch
                let specifier = module_specifier.clone();
                let client = self.http_client.clone();
                let source_maps = self.source_maps.clone();

                let fut = async move {
                    let requested_specifier = specifier.clone();

                    let specifier = if specifier.scheme() == "jsr" {
                        resolve_jsr_specifier(specifier.as_str())?
                    } else {
                        specifier
                    };

                    let specifier = if is_jsr_package_entry_url(&specifier) {
                        resolve_jsr_entry_to_module(&client, &specifier).await?
                    } else {
                        specifier
                    };

                    let response = client
                        .get(specifier.as_str())
                        .header("Accept", "application/typescript,application/javascript,text/typescript,text/javascript,*/*")
                        .send()
                        .await
                        .map_err(|e| {
                            JsErrorBox::generic(format!(
                                "Failed to fetch '{}': {}",
                                specifier, e
                            ))
                        })?;

                    // Follow redirects — reqwest does this by default, but capture the final URL
                    let final_url = response.url().clone();
                    let content_type = response
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();

                    if !response.status().is_success() {
                        return Err(JsErrorBox::generic(format!(
                            "HTTP {} fetching '{}'",
                            response.status(),
                            specifier
                        )));
                    }

                    let code = response.text().await.map_err(|e| {
                        JsErrorBox::generic(format!(
                            "Failed to read response from '{}': {}",
                            specifier, e
                        ))
                    })?;

                    // Determine media type from Content-Type header or URL extension
                    let final_specifier =
                        ModuleSpecifier::parse(final_url.as_str()).unwrap_or(specifier.clone());
                    let media_type = media_type_from_content_type(&content_type, &final_specifier);

                    let (module_type, should_transpile) = match media_type {
                        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                            (ModuleType::JavaScript, false)
                        }
                        MediaType::Jsx => (ModuleType::JavaScript, true),
                        MediaType::TypeScript
                        | MediaType::Mts
                        | MediaType::Cts
                        | MediaType::Dts
                        | MediaType::Dmts
                        | MediaType::Dcts
                        | MediaType::Tsx => (ModuleType::JavaScript, true),
                        MediaType::Json => (ModuleType::Json, false),
                        _ => {
                            // Default to JS for remote modules with unknown type
                            (ModuleType::JavaScript, false)
                        }
                    };

                    let code = if should_transpile {
                        transpile(&final_specifier, code, media_type, &source_maps)?
                    } else {
                        code
                    };

                    if requested_specifier.as_str() != final_specifier.as_str() {
                        Ok(ModuleSource::new_with_redirect(
                            module_type,
                            ModuleSourceCode::String(code.into()),
                            &requested_specifier,
                            &final_specifier,
                            None,
                        ))
                    } else if final_url.as_str() != specifier.as_str() {
                        Ok(ModuleSource::new_with_redirect(
                            module_type,
                            ModuleSourceCode::String(code.into()),
                            &specifier,
                            &final_specifier,
                            None,
                        ))
                    } else {
                        Ok(ModuleSource::new(
                            module_type,
                            ModuleSourceCode::String(code.into()),
                            &specifier,
                            None,
                        ))
                    }
                };

                ModuleLoadResponse::Async(Pin::from(Box::new(fut)))
            }
            _ => ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
                "Unsupported module scheme: '{}' in '{}'",
                scheme, module_specifier
            )))),
        }
    }

    fn get_source_map(&self, specifier: &str) -> Option<Cow<'_, [u8]>> {
        self.source_maps
            .borrow()
            .get(specifier)
            .map(|v| v.clone().into())
    }
}
