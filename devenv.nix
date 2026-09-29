{
  pkgs,
  # lib,
  # config,
  # inputs,
  ...
}:

{
  # https://devenv.sh/packages/
  packages = [ pkgs.git ];

  languages.rust = {
    enable = true;
    channel = "stable"; # default
  };
}
