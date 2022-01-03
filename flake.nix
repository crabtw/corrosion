{
  outputs = { self, nixpkgs }:
    let

      systems = [
        "x86_64-linux"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);

      buildPackage = { system }:
        let

          pkgs = import nixpkgs {
            inherit system;
          };

        in pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            clippy
            cmake
            ninja
          ];
        };

    in {
      devShell = forAllSystems (system:
        buildPackage { inherit system; }
      );
    };
}
