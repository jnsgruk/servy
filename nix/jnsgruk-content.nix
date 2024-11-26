{
  fetchFromGitHub,
  hugo,
  lib,
  stdenv,
}:

stdenv.mkDerivation rec {
  pname = "jnsgruk-site";
  version = "25a0f26e";

  src = fetchFromGitHub {
    owner = "jnsgruk";
    repo = "jnsgr.uk";
    rev = version;
    hash = "sha256-FWZpPej8FD2ZoijO1HUU0BeHSt5aYZzRapHBNSGOcnc=";
  };

  buildInputs = [ hugo ];

  # Nix doesn't play well with Hugo's "GitInfo" module, so disable it and inject
  # the revision from the flake.
  postPatch = ''
    substituteInPlace ./site/layouts/shortcodes/gitinfo.html \
      --replace-fail "{{ .Page.GitInfo.Hash }}" "${version}" \
      --replace-fail "{{ .Page.GitInfo.AbbreviatedHash }}" "${version}"

    substituteInPlace ./site/config/_default/config.yaml \
      --replace-fail "enableGitInfo: true" "enableGitInfo: false"
  '';

  buildPhase = ''
    runHook preBuild
    hugo --minify -s site -d ../public
    runHook postBuild
  '';

  installPhase = ''
    mkdir -p $out/share/jnsgruk/
    mv ./public $out/share/jnsgruk/public
  '';

  meta = {
    description = "Personal homepage content for jnsgr.uk";
    homepage = "https://github.com/jnsgruk/jnsgr.uk";
    license = lib.licenses.asl20;
    platforms = lib.platforms.all;
    maintainers = with lib.maintainers; [ jnsgruk ];
  };
}
