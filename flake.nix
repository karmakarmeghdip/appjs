{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:nixos/nixpkgs";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };

        # User specified Rust version
        msrvRustVersion = "1.93.1";
        
        # Ensure semantic versioning format for rust-bin (e.g. "1.88" -> "1.88.0")
        rustOverlayVersion = if builtins.length (builtins.split "\\." msrvRustVersion) == 3 
          then "${msrvRustVersion}.0" 
          else msrvRustVersion;
        
        rustToolchain = pkgs.rust-bin.stable."${rustOverlayVersion}".default.override {
          extensions = [ "rustfmt" "rust-analyzer" "rust-src" ];
          targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" "x86_64-pc-windows-gnu" "aarch64-linux-android" ];
        };

        commonPackages = [
          rustToolchain
        ];

        masonryPackages = with pkgs; commonPackages ++ [
          pkg-config
          clang
          llvmPackages.libclang

          fontconfig

          libxkbcommon
          libxcb
          libx11
          libxcursor
          libxi
          libxrandr
          libxxf86vm

          vulkan-loader

          wayland
          wayland-protocols
          wayland-scanner
        ];

        mkDevShell = packages: pkgs.mkShell {
          inherit packages;
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packages}";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      in
      {
        devShells.default = mkDevShell masonryPackages;
        
        # Legacy alias if needed, but 'default' is primary
        devShells.masonry = mkDevShell masonryPackages;
      });
}
