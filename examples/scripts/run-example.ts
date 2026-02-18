const inputPath = Bun.argv[2];

async function main(): Promise<void> {
    if (!inputPath) {
        console.error("Usage: bun run run:example <path-to-entry-file>");
        process.exit(1);
    }

    const buildResult = await Bun.build({
        entrypoints: [inputPath],
        outdir: "./dist",
        format: "iife",
        target: "browser",
        sourcemap: "external",
    });

    if (!buildResult.success) {
        for (const log of buildResult.logs) {
            console.error(log);
        }
        process.exit(1);
    }

    const outputFile = buildResult.outputs
        .map((output) => output.path)
        .find((outputPath) => outputPath.endsWith(".js") && !outputPath.endsWith(".js.map"))
        ?.split("/")
        .pop();

    if (!outputFile) {
        console.error("Build succeeded but could not determine generated bundle path.");
        process.exit(1);
    }

    const runResult = Bun.spawnSync({
        cmd: ["cargo", "run", "--", `../examples/dist/${outputFile}`],
        stdout: "inherit",
        stderr: "inherit",
    });

    process.exit(runResult.exitCode ?? 1);
}

void main();
