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
}
