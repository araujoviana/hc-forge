export function shellSingleQuote(value) {
  return "'" + String(value ?? "").replace(/'/g, `'"'"'`) + "'";
}

export const DEFAULT_PLATFORM_DOCKERFILE_PATH = "/root/hcforge/docker/Dockerfile";

function renderDockerInstallSnippet(flagVarName = "INSTALL_DOCKER") {
  const snippet = `
if ! command -v docker >/dev/null 2>&1; then
  if [ "$INSTALL_DOCKER" != "1" ]; then
    echo "Docker is not installed and installation is disabled."
    exit 15
  fi

  if command -v apt-get >/dev/null 2>&1; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y ca-certificates curl gnupg lsb-release docker.io
  elif command -v dnf >/dev/null 2>&1; then
    dnf -y install docker || dnf -y install moby-engine
  elif command -v yum >/dev/null 2>&1; then
    yum -y install docker || yum -y install moby-engine
  elif command -v zypper >/dev/null 2>&1; then
    zypper --non-interactive refresh
    zypper --non-interactive install -y docker
  elif command -v pacman >/dev/null 2>&1; then
    pacman -Syu --noconfirm docker
  elif command -v apk >/dev/null 2>&1; then
    apk update
    apk add docker docker-cli containerd openrc
  else
    echo "No supported package manager found to install Docker."
    exit 16
  fi
fi

if command -v systemctl >/dev/null 2>&1; then
  systemctl enable --now docker || true
fi
if command -v service >/dev/null 2>&1; then
  service docker start || true
fi
if command -v rc-service >/dev/null 2>&1; then
  rc-service docker start || true
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker command is still unavailable after installation steps."
  exit 17
fi
`;
  return snippet.replaceAll("INSTALL_DOCKER", flagVarName);
}

function normalizeNumber(value, fallback, min, max) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return fallback;
  }
  return Math.min(max, Math.max(min, Math.trunc(parsed)));
}

function normalizeNixPackageToken(token) {
  const normalized = String(token ?? "").trim();
  if (!normalized) {
    return "";
  }
  return normalized.replace(/[^A-Za-z0-9._+-]/g, "");
}

function normalizeNixPackageList(value) {
  return String(value ?? "")
    .split(/[\s,]+/)
    .map((token) => normalizeNixPackageToken(token))
    .filter(Boolean)
    .slice(0, 80);
}

function parseJsonLines(stdout) {
  const lines = String(stdout ?? "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  const parsed = [];
  for (const line of lines) {
    try {
      const item = JSON.parse(line);
      if (item && typeof item === "object") {
        parsed.push(item);
      }
    } catch {
      // Ignore malformed lines and keep best-effort parsing.
    }
  }
  return parsed;
}

function asString(value, fallback = "—") {
  if (value == null) {
    return fallback;
  }
  const text = String(value).trim();
  return text || fallback;
}

function firstNonEmpty(...values) {
  for (const value of values) {
    const text = String(value ?? "").trim();
    if (text) {
      return text;
    }
  }
  return "";
}

function renderNixPathBootstrapSnippet() {
  return `
if [ -f /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]; then
  . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
fi
if [ -f "$HOME/.nix-profile/etc/profile.d/nix.sh" ]; then
  . "$HOME/.nix-profile/etc/profile.d/nix.sh"
fi
export PATH="/nix/var/nix/profiles/default/bin:$HOME/.nix-profile/bin:$PATH"
`.trim();
}

function utf8ToBase64(value) {
  const normalized = String(value ?? "");
  if (typeof Buffer !== "undefined") {
    return Buffer.from(normalized, "utf8").toString("base64");
  }
  if (typeof TextEncoder !== "undefined" && typeof btoa === "function") {
    const bytes = new TextEncoder().encode(normalized);
    let binary = "";
    const chunkSize = 0x8000;
    for (let index = 0; index < bytes.length; index += chunkSize) {
      const chunk = bytes.subarray(index, index + chunkSize);
      binary += String.fromCharCode(...chunk);
    }
    return btoa(binary);
  }
  throw new Error("Unable to base64-encode Dockerfile content in this runtime.");
}

export function buildDockerSetupCommand(options = {}) {
  const installDocker = options.installDocker !== false;
  const dockerfileContent =
    typeof options.dockerfileContent === "string"
      ? options.dockerfileContent.replace(/\r\n/g, "\n")
      : "";
  const shouldUploadDockerfile = dockerfileContent.trim().length > 0;
  const dockerfileTargetPath =
    String(options.dockerfileTargetPath ?? DEFAULT_PLATFORM_DOCKERFILE_PATH).trim() ||
    DEFAULT_PLATFORM_DOCKERFILE_PATH;
  const dockerfileBase64 = shouldUploadDockerfile ? utf8ToBase64(dockerfileContent) : "";
  return `
set -eu
INSTALL_DOCKER=${installDocker ? "1" : "0"}
${renderDockerInstallSnippet("INSTALL_DOCKER")}
${shouldUploadDockerfile ? `DOCKERFILE_TARGET_PATH=${shellSingleQuote(dockerfileTargetPath)}
DOCKERFILE_B64=${shellSingleQuote(dockerfileBase64)}
mkdir -p "$(dirname "$DOCKERFILE_TARGET_PATH")"
if ! command -v base64 >/dev/null 2>&1; then
  echo "base64 command is required to upload Dockerfile content."
  exit 18
fi
(printf '%s' "$DOCKERFILE_B64" | base64 -d > "$DOCKERFILE_TARGET_PATH" 2>/dev/null) || (printf '%s' "$DOCKERFILE_B64" | base64 --decode > "$DOCKERFILE_TARGET_PATH" 2>/dev/null) || {
  echo "Failed to decode Dockerfile content on target host."
  exit 19
}
chmod 600 "$DOCKERFILE_TARGET_PATH" || true
echo "Dockerfile uploaded to $DOCKERFILE_TARGET_PATH"` : ""}
docker --version
`.trim();
}

export function buildDockerImagesCommand() {
  return `
set -eu
command -v docker >/dev/null 2>&1 || { echo "Docker is not installed."; exit 20; }
docker images --format '{{json .}}'
`.trim();
}

export function buildDockerContainersCommand() {
  return `
set -eu
command -v docker >/dev/null 2>&1 || { echo "Docker is not installed."; exit 21; }
docker ps -a --format '{{json .}}'
`.trim();
}

export function parseDockerImages(stdout) {
  return parseJsonLines(stdout).map((item) => ({
    repository: asString(item.Repository, "<none>"),
    tag: asString(item.Tag, "<none>"),
    id: asString(item.ID),
    createdSince: asString(item.CreatedSince),
    size: asString(item.Size),
  }));
}

export function parseDockerContainers(stdout) {
  return parseJsonLines(stdout).map((item) => ({
    name: asString(item.Names),
    image: asString(item.Image),
    status: asString(item.Status, asString(item.State)),
    ports: asString(item.Ports, "none"),
    id: asString(item.ID),
  }));
}

export function buildDockerfileTemplate(options = {}) {
  const baseImage = String(options.baseImage ?? "ubuntu:24.04").trim() || "ubuntu:24.04";
  const workdir = String(options.workdir ?? "/app").trim() || "/app";
  const exposePort = String(options.exposePort ?? "").trim();
  const startCommand = String(options.startCommand ?? '["bash"]').trim() || '["bash"]';

  let cmdLine = `CMD ${startCommand}`;
  if (!(startCommand.startsWith("[") && startCommand.endsWith("]"))) {
    const parts = startCommand
      .split(/\s+/)
      .map((part) => part.trim())
      .filter(Boolean);
    cmdLine = `CMD ${JSON.stringify(parts.length ? parts : ["bash"])}`;
  }

  const lines = [
    "# Generated by HC Forge Platform Ops",
    `FROM ${baseImage}`,
    `WORKDIR ${workdir}`,
    "COPY . .",
  ];
  if (exposePort) {
    lines.push(`EXPOSE ${exposePort}`);
  }
  lines.push(cmdLine);
  return lines.join("\n") + "\n";
}

export function parseDockerfileTemplate(content) {
  const lines = String(content ?? "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("#"));

  let baseImage;
  let workdir;
  let exposePort;
  let startCommand;

  for (const line of lines) {
    const fromMatch = line.match(/^FROM\s+(.+)$/i);
    if (fromMatch) {
      const raw = fromMatch[1].trim();
      const stageSplit = raw.split(/\s+AS\s+/i);
      baseImage = (stageSplit[0] ?? raw).trim();
      continue;
    }

    const workdirMatch = line.match(/^WORKDIR\s+(.+)$/i);
    if (workdirMatch) {
      workdir = workdirMatch[1].trim();
      continue;
    }

    const exposeMatch = line.match(/^EXPOSE\s+(.+)$/i);
    if (exposeMatch) {
      const token = exposeMatch[1].trim().split(/\s+/)[0];
      if (token) {
        exposePort = token;
      }
      continue;
    }

    const cmdMatch = line.match(/^CMD\s+(.+)$/i);
    if (cmdMatch) {
      startCommand = cmdMatch[1].trim();
    }
  }

  return {
    baseImage,
    workdir,
    exposePort,
    startCommand,
  };
}

export function buildMinikubeSetupCommand(options = {}) {
  const installMinikube = options.installMinikube !== false;
  const ensureDocker = options.ensureDocker !== false;
  const autoStart = options.autoStart !== false;
  const profile = String(options.profile ?? "hcforge").trim() || "hcforge";
  const driver = options.driver === "none" ? "none" : "docker";
  const cpus = normalizeNumber(options.cpus, 2, 1, 64);
  const memoryMb = normalizeNumber(options.memoryMb, 4096, 1024, 262144);
  const kubernetesVersion = String(options.kubernetesVersion ?? "").trim();

  return `
set -eu
PROFILE=${shellSingleQuote(profile)}
MINIKUBE_DRIVER=${shellSingleQuote(driver)}
MINIKUBE_CPUS=${cpus}
MINIKUBE_MEMORY_MB=${memoryMb}
MINIKUBE_K8S_VERSION=${shellSingleQuote(kubernetesVersion)}
MINIKUBE_AUTO_START=${autoStart ? "1" : "0"}
MINIKUBE_INSTALL=${installMinikube ? "1" : "0"}
MINIKUBE_ENSURE_DOCKER=${ensureDocker ? "1" : "0"}

if ! command -v curl >/dev/null 2>&1; then
  if command -v apt-get >/dev/null 2>&1; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y curl
  elif command -v dnf >/dev/null 2>&1; then
    dnf -y install curl
  elif command -v yum >/dev/null 2>&1; then
    yum -y install curl
  elif command -v zypper >/dev/null 2>&1; then
    zypper --non-interactive refresh
    zypper --non-interactive install -y curl
  elif command -v pacman >/dev/null 2>&1; then
    pacman -Syu --noconfirm curl
  elif command -v apk >/dev/null 2>&1; then
    apk update
    apk add curl
  else
    echo "curl is required but no supported package manager is available."
    exit 30
  fi
fi

if [ "$MINIKUBE_DRIVER" = "docker" ] && [ "$MINIKUBE_ENSURE_DOCKER" = "1" ] && ! command -v docker >/dev/null 2>&1; then
  INSTALL_DOCKER=1
  ${renderDockerInstallSnippet("INSTALL_DOCKER")}
fi

if [ "$MINIKUBE_DRIVER" = "docker" ] && ! command -v docker >/dev/null 2>&1; then
  echo "Docker driver selected but docker command is unavailable."
  exit 31
fi

if ! command -v minikube >/dev/null 2>&1; then
  if [ "$MINIKUBE_INSTALL" != "1" ]; then
    echo "Minikube is not installed and installation is disabled."
    exit 32
  fi

  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64|amd64)
      BIN_ARCH="amd64"
      ;;
    aarch64|arm64)
      BIN_ARCH="arm64"
      ;;
    *)
      echo "Unsupported architecture for minikube: $ARCH"
      exit 33
      ;;
  esac

  curl -fsSL "https://storage.googleapis.com/minikube/releases/latest/minikube-linux-$BIN_ARCH" -o /usr/local/bin/minikube
  chmod +x /usr/local/bin/minikube

  if ! command -v kubectl >/dev/null 2>&1; then
    K8S_STABLE="$(curl -fsSL https://dl.k8s.io/release/stable.txt)"
    curl -fsSL "https://dl.k8s.io/release/$K8S_STABLE/bin/linux/$BIN_ARCH/kubectl" -o /usr/local/bin/kubectl
    chmod +x /usr/local/bin/kubectl
  fi
fi

if [ "$MINIKUBE_AUTO_START" = "1" ]; then
  set -- start -p "$PROFILE" --driver="$MINIKUBE_DRIVER" --cpus="$MINIKUBE_CPUS" --memory="$MINIKUBE_MEMORY_MB"
  if [ -n "$MINIKUBE_K8S_VERSION" ]; then
    set -- "$@" --kubernetes-version="$MINIKUBE_K8S_VERSION"
  fi
  minikube "$@"
fi

minikube status -p "$PROFILE" --output=json || minikube status -p "$PROFILE" || true
`.trim();
}

export function buildMinikubeStatusCommand(profile) {
  const safeProfile = String(profile ?? "").trim() || "hcforge";
  return `
set -eu
PROFILE=${shellSingleQuote(safeProfile)}
command -v minikube >/dev/null 2>&1 || { echo "minikube is not installed."; exit 40; }
minikube status -p "$PROFILE" --output=json || minikube status -p "$PROFILE" || true
`.trim();
}

export function buildMinikubeNodesCommand(profile) {
  const safeProfile = String(profile ?? "").trim() || "hcforge";
  return `
set -eu
PROFILE=${shellSingleQuote(safeProfile)}
command -v minikube >/dev/null 2>&1 || { echo "minikube is not installed."; exit 41; }
minikube kubectl -p "$PROFILE" -- get nodes -o wide
`.trim();
}

export function buildMinikubePodsCommand(profile) {
  const safeProfile = String(profile ?? "").trim() || "hcforge";
  return `
set -eu
PROFILE=${shellSingleQuote(safeProfile)}
command -v minikube >/dev/null 2>&1 || { echo "minikube is not installed."; exit 42; }
minikube kubectl -p "$PROFILE" -- get pods -A -o wide
`.trim();
}

export function buildNixSetupCommand(options = {}) {
  const installNix = options.installNix !== false;
  const enableFlakes = options.enableFlakes !== false;
  const runGarbageCollect = options.runGarbageCollect === true;
  const packages = normalizeNixPackageList(options.packages);
  const packagesValue = packages.join(" ");
  const pathBootstrap = renderNixPathBootstrapSnippet();

  return `
set -eu
NIX_INSTALL=${installNix ? "1" : "0"}
NIX_ENABLE_FLAKES=${enableFlakes ? "1" : "0"}
NIX_RUN_GC=${runGarbageCollect ? "1" : "0"}
NIX_PACKAGES=${shellSingleQuote(packagesValue)}

if ! command -v nix >/dev/null 2>&1 && ! command -v nix-env >/dev/null 2>&1; then
  if [ "$NIX_INSTALL" != "1" ]; then
    echo "Nix is not installed and installation is disabled."
    exit 50
  fi

  # Prefer distro packages first when available (faster and simpler on Ubuntu).
  if command -v apt-get >/dev/null 2>&1; then
    export DEBIAN_FRONTEND=noninteractive
    (apt-get update && apt-get install -y nix-bin) || true
  elif command -v dnf >/dev/null 2>&1; then
    dnf -y install nix || true
  elif command -v yum >/dev/null 2>&1; then
    yum -y install nix || true
  elif command -v zypper >/dev/null 2>&1; then
    (zypper --non-interactive refresh && zypper --non-interactive install -y nix) || true
  elif command -v pacman >/dev/null 2>&1; then
    pacman -Syu --noconfirm nix || true
  elif command -v apk >/dev/null 2>&1; then
    (apk update && apk add nix) || true
  fi

  if command -v apt-get >/dev/null 2>&1; then
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y curl xz-utils
  elif command -v dnf >/dev/null 2>&1; then
    dnf -y install curl xz
  elif command -v yum >/dev/null 2>&1; then
    yum -y install curl xz
  elif command -v zypper >/dev/null 2>&1; then
    zypper --non-interactive refresh
    zypper --non-interactive install -y curl xz
  elif command -v pacman >/dev/null 2>&1; then
    pacman -Syu --noconfirm curl xz
  elif command -v apk >/dev/null 2>&1; then
    apk update
    apk add curl xz
  fi

  if ! command -v nix >/dev/null 2>&1 && ! command -v nix-env >/dev/null 2>&1; then
    curl -fsSL https://nixos.org/nix/install -o /tmp/hcforge-nix-install.sh
    sh /tmp/hcforge-nix-install.sh --no-daemon --yes
  fi
fi

${pathBootstrap}

if ! command -v nix >/dev/null 2>&1 && ! command -v nix-env >/dev/null 2>&1; then
  echo "nix command is unavailable after installation."
  exit 51
fi

if [ "$NIX_ENABLE_FLAKES" = "1" ] && command -v nix >/dev/null 2>&1; then
  mkdir -p /etc/nix
  touch /etc/nix/nix.conf
  if grep -Eq '^[[:space:]]*experimental-features[[:space:]]*=' /etc/nix/nix.conf; then
    sed -i 's/^[[:space:]]*experimental-features[[:space:]]*=.*/experimental-features = nix-command flakes/' /etc/nix/nix.conf
  else
    printf '\\nexperimental-features = nix-command flakes\\n' >> /etc/nix/nix.conf
  fi
fi

if [ -n "$NIX_PACKAGES" ]; then
  if command -v nix-env >/dev/null 2>&1; then
    if command -v nix-channel >/dev/null 2>&1; then
      nix-channel --list | grep -q '^nixpkgs ' || nix-channel --add https://nixos.org/channels/nixpkgs-unstable nixpkgs
      nix-channel --update nixpkgs || true
    fi
    for package in $NIX_PACKAGES; do
      nix-env -iA "nixpkgs.$package"
    done
  elif command -v nix >/dev/null 2>&1; then
    for package in $NIX_PACKAGES; do
      nix profile install --accept-flake-config "nixpkgs#$package"
    done
  fi
fi

if [ "$NIX_RUN_GC" = "1" ]; then
  if command -v nix-collect-garbage >/dev/null 2>&1; then
    nix-collect-garbage -d || true
  elif command -v nix >/dev/null 2>&1; then
    nix store gc || true
  fi
fi

if command -v nix >/dev/null 2>&1; then
  nix --version
elif command -v nix-env >/dev/null 2>&1; then
  nix-env --version
fi
`.trim();
}

export function buildNixVersionCommand() {
  const pathBootstrap = renderNixPathBootstrapSnippet();
  return `
set -eu
${pathBootstrap}
if command -v nix >/dev/null 2>&1; then
  nix --version
  nix show-config 2>/dev/null | grep -E '^experimental-features\\b' || true
elif command -v nix-env >/dev/null 2>&1; then
  nix-env --version
else
  echo "nix is not installed."
  exit 52
fi
`.trim();
}

export function buildNixPackagesCommand() {
  const pathBootstrap = renderNixPathBootstrapSnippet();
  return `
set -eu
${pathBootstrap}
if command -v nix >/dev/null 2>&1; then
  nix profile list --json 2>/dev/null || nix profile list 2>/dev/null || true
elif command -v nix-env >/dev/null 2>&1; then
  nix-env -q --installed
else
  echo "nix is not installed."
  exit 53
fi
`.trim();
}

export function buildNixStoreUsageCommand() {
  const pathBootstrap = renderNixPathBootstrapSnippet();
  return `
set -eu
${pathBootstrap}
if command -v nix-store >/dev/null 2>&1; then
  echo "Live store bytes:"
  nix-store --gc --print-live
  echo "Dead store bytes:"
  nix-store --gc --print-dead
  if [ -d /nix/store ] && command -v du >/dev/null 2>&1; then
    echo "Filesystem usage for /nix/store:"
    du -sh /nix/store 2>/dev/null || true
  fi
elif command -v nix >/dev/null 2>&1; then
  nix store info || true
else
  echo "nix is not installed."
  exit 54
fi
`.trim();
}

function normalizeParsedNixPackages(items) {
  const normalized = [];
  for (const [index, item] of items.entries()) {
    if (!item || typeof item !== "object") {
      continue;
    }
    const attrPath = firstNonEmpty(item.attrPath, item.name, item.pname);
    const derivedName = attrPath.includes(".")
      ? (attrPath.split(".").pop() ?? attrPath).trim()
      : attrPath;
    const fallbackName = `package-${index + 1}`;
    const name = firstNonEmpty(derivedName, attrPath, fallbackName);
    const version = firstNonEmpty(
      item.version,
      item.versionedName,
      item.locked?.rev,
      item.locked?.lastModified,
      "—"
    );
    const source = firstNonEmpty(
      item.originalUrl,
      item.url,
      item.flake,
      Array.isArray(item.storePaths) ? item.storePaths[0] : "",
      "nix profile"
    );
    normalized.push({
      name,
      version,
      source,
    });
  }
  return normalized;
}

export function parseNixPackages(output) {
  const text = String(output ?? "").trim();
  if (!text) {
    return [];
  }
  if (/^nix is not installed\.?$/i.test(text)) {
    return [];
  }

  try {
    const parsed = JSON.parse(text);
    if (Array.isArray(parsed)) {
      return normalizeParsedNixPackages(parsed);
    }
    if (parsed && typeof parsed === "object") {
      if (Array.isArray(parsed.elements)) {
        return normalizeParsedNixPackages(parsed.elements);
      }
      return normalizeParsedNixPackages(Object.values(parsed));
    }
  } catch {
    // Fall through to plain text parsing.
  }

  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line && !/^nix is not installed\.?$/i.test(line))
    .map((line) => {
      const match = line.match(/^(.+)-([0-9][A-Za-z0-9.+-]*)$/);
      if (match) {
        return {
          name: match[1].trim(),
          version: match[2].trim(),
          source: "nix-env",
        };
      }
      return {
        name: line,
        version: "—",
        source: "nix",
      };
    });
}
