{ pkgs }: rec {
  extract-messages = pkgs.writeShellScriptBin "extract-messages" ''
    set -e

    SOURCE_POT=locales/source.pot

    echo "Extracting new string templates."
    export SOURCE_DATE_EPOCH=1
    ${pkgs.gettext}/bin/xgettext \
      --from-code=UTF-8 \
      --add-comments=TRANSLATOR \
      --language=C \
      --package-version="0.1.0" \
      --no-location \
      --no-wrap \
      --sort-output \
      -o ''${SOURCE_POT} \
      "$@" \
      2>&1 | grep -v "warning: unterminated character constant" \
      || true

    # Remove the date to make output more deterministic.
    ${pkgs.gnused}/bin/sed -i '/"POT-Creation-Date:/d' ''${SOURCE_POT}
  '';
}
