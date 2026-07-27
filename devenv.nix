{ pkgs, lib, config, inputs, ... }:

{
  name = "cebelca-cli";
  packages = [ pkgs.git ];
  
  languages.rust = {
    enable = true;
    channel = "stable";
    version = "1.97.1";
    lsp.enable = true;
  };

  env = {
    RUST_BACKTRACE = "full";
    NIX_ENFORCE_PURITY = 0;
  };

  # Put the release binary on PATH so `ceb` runs it. An `alias` won't work here:
  # the `.envrc` enters via direnv (`use devenv`), which only propagates env vars,
  # not shell aliases. Build the binary first with `cargo build --release`.
  enterShell = ''
    export PATH="$DEVENV_ROOT/target/release:$PATH"
  '';
}
