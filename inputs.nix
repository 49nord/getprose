{ pkgs, rust }: rec {
  scripts = import ./scripts.nix { pkgs = pkgs; };

  nativeBuildInputs = with pkgs; [
    rust

    # i18n
    gettext
    scripts.extract-messages
  ];

  buildInputs = [ pkgs.gettext ];
}
