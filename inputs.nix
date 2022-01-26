{ pkgs, rust }: rec {
  nativeBuildInputs = [ rust pkgs.gettext ];
  buildInputs = [ pkgs.gettext ];
}
