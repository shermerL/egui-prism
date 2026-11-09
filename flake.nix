{
  description = "Development tools for egui-prism";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/d3b13ad75cc492690728f9307e443dec9ef9a20e";
  inputs.quest.url = "github:moq-dev/quest/0c1da19a74f77e4f662cf9536fb789aca686b3bc";

  outputs =
    { nixpkgs, quest, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = eachSystem (system: {
        default = quest.packages.${system}.default;
      });
      devShells = eachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = [
              quest.packages.${system}.default
              pkgs.rustup
              pkgs.nodejs_24
              pkgs.python3
              pkgs.nixfmt
              pkgs.actionlint
              pkgs.shellcheck
              pkgs.pkg-config
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
              pkgs.libxkbcommon
              pkgs.wayland
              pkgs.libGL
              pkgs.vulkan-loader
            ];
            env = pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
                pkgs.libxkbcommon
                pkgs.wayland
                pkgs.libGL
                pkgs.vulkan-loader
                pkgs.xorg.libX11
                pkgs.xorg.libXcursor
                pkgs.xorg.libXi
                pkgs.xorg.libXrandr
              ];
            };
          };
        }
      );
    };
}
