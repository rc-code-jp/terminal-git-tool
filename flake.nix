{
  description = "A compact git TUI for narrow terminal panes";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          pocogit = pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            src = pkgs.lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                pkgs.lib.cleanSourceFilter path type && builtins.baseNameOf path != "target";
            };
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [
              pkgs.git
              pkgs.makeWrapper
            ];

            buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin (
              [
                pkgs.apple-sdk_15
              ]
            );

            postInstall = ''
              wrapProgram "$out/bin/pocogit" \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.git ]}
            '';

            meta = {
              description = cargoToml.package.description;
              homepage = "https://github.com/rc-code-jp/pocogit";
              license = pkgs.lib.licenses.mit;
              mainProgram = "pocogit";
              platforms = systems;
            };
          };
        in
        {
          default = pocogit;
          pocogit = pocogit;
        }
      );

      apps = forAllSystems (
        system:
        let
          program = "${self.packages.${system}.pocogit}/bin/pocogit";
        in
        {
          default = {
            type = "app";
            inherit program;
          };

          pocogit = {
            type = "app";
            inherit program;
          };
        }
      );
    };
}
