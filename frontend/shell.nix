{ pkgs 
}:

with pkgs;

mkShellNoCC {
  nativeBuildInputs = [
    nodejs-18_x
  ];
}
