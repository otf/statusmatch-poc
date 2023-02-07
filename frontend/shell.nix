{ pkgs 
}:

with pkgs;

mkShellNoCC {
  nativeBuildInputs = [
    nodejs-19_x
  ];
}
