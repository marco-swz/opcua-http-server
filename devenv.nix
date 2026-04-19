{ pkgs, lib, config, inputs, ... }:

{
  env.GREET = "devenv";

  packages = with pkgs; [ 
        git
        bashInteractive
  ];

  languages.rust.enable = true;
}
