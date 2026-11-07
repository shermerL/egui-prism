{
  description = "Development tools for egui-prism";

  inputs.nixpkgs.follows = "quest/nixpkgs";
  inputs.quest.url = "github:shermerL/quest/19a2438cd80cf968dddfbf664570a2c8b9a254bf";

  outputs =
    { quest, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      eachSystem = quest.inputs.nixpkgs.lib.genAttrs systems;
    in
    {
      packages = eachSystem (system: {
        default = quest.packages.${system}.default;
      });
      devShells = eachSystem (
        system:
        let
          pkgs = import quest.inputs.nixpkgs { inherit system; };
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
