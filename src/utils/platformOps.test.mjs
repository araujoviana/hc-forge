import test from "node:test";
import assert from "node:assert/strict";

import {
  buildNixPackagesCommand,
  buildNixSetupCommand,
  buildNixStoreUsageCommand,
  buildNixVersionCommand,
  DEFAULT_PLATFORM_DOCKERFILE_PATH,
  buildDockerContainersCommand,
  buildDockerSetupCommand,
  buildDockerfileTemplate,
  buildMinikubeSetupCommand,
  parseNixPackages,
  parseDockerfileTemplate,
  parseDockerContainers,
  parseDockerImages,
} from "./platformOps.js";

test("platformOps command builders and parsers behave as expected", () => {
  const imagesOutput = [
    '{"Repository":"nginx","Tag":"latest","ID":"sha256:123","CreatedSince":"2 weeks ago","Size":"187MB"}',
    '{"Repository":"redis","Tag":"7","ID":"sha256:456","CreatedSince":"4 days ago","Size":"52MB"}',
  ].join("\n");
  const parsedImages = parseDockerImages(imagesOutput);
  assert.equal(parsedImages.length, 2);
  assert.equal(parsedImages[0].repository, "nginx");
  assert.equal(parsedImages[1].tag, "7");

  const containersOutput =
    '{"ID":"a1b2c3","Image":"nginx:latest","Status":"Up 2 hours","Ports":"80/tcp","Names":"web"}';
  const parsedContainers = parseDockerContainers(containersOutput);
  assert.equal(parsedContainers.length, 1);
  assert.equal(parsedContainers[0].name, "web");
  assert.equal(parsedContainers[0].status, "Up 2 hours");

  const dockerfile = buildDockerfileTemplate({
    baseImage: "node:22-alpine",
    workdir: "/srv/app",
    exposePort: "3000",
    startCommand: '["node","server.js"]',
  });
  assert.match(dockerfile, /FROM node:22-alpine/);
  assert.match(dockerfile, /WORKDIR \/srv\/app/);
  assert.match(dockerfile, /EXPOSE 3000/);
  assert.match(dockerfile, /CMD \["node","server\.js"\]/);

  const parsedDockerfile = parseDockerfileTemplate(`
    FROM golang:1.23 AS builder
    WORKDIR /workspace
    EXPOSE 8080
    CMD ["go", "run", "./cmd/app"]
  `);
  assert.equal(parsedDockerfile.baseImage, "golang:1.23");
  assert.equal(parsedDockerfile.workdir, "/workspace");
  assert.equal(parsedDockerfile.exposePort, "8080");
  assert.equal(parsedDockerfile.startCommand, '["go", "run", "./cmd/app"]');

  const withInstall = buildDockerSetupCommand({ installDocker: true });
  const withoutInstall = buildDockerSetupCommand({ installDocker: false });
  assert.match(withInstall, /INSTALL_DOCKER=1/);
  assert.match(withoutInstall, /INSTALL_DOCKER=0/);
  assert.doesNotMatch(withInstall, /DOCKERFILE_B64=/);

  const withDockerfile = buildDockerSetupCommand({
    installDocker: true,
    dockerfileContent: "FROM alpine:3.20\nCMD [\"echo\",\"hi\"]\n",
    dockerfileTargetPath: "/tmp/demo/Dockerfile",
  });
  assert.match(withDockerfile, /DOCKERFILE_TARGET_PATH='\/tmp\/demo\/Dockerfile'/);
  assert.match(withDockerfile, /DOCKERFILE_B64='/);
  assert.match(withDockerfile, /Dockerfile uploaded to \$DOCKERFILE_TARGET_PATH/);
  assert.match(withDockerfile, /base64 -d/);
  assert.match(withDockerfile, /base64 --decode/);

  const withDefaultTarget = buildDockerSetupCommand({
    dockerfileContent: "FROM scratch\n",
  });
  assert.match(
    withDefaultTarget,
    new RegExp(
      `DOCKERFILE_TARGET_PATH='${DEFAULT_PLATFORM_DOCKERFILE_PATH.replaceAll("/", "\\/")}'`
    )
  );

  const minikubeCommand = buildMinikubeSetupCommand({
    installMinikube: true,
    ensureDocker: true,
    autoStart: true,
    profile: "demo-profile",
    driver: "none",
    cpus: 4,
    memoryMb: 8192,
    kubernetesVersion: "v1.31.0",
  });
  assert.match(minikubeCommand, /PROFILE='demo-profile'/);
  assert.match(minikubeCommand, /MINIKUBE_DRIVER='none'/);
  assert.match(minikubeCommand, /MINIKUBE_CPUS=4/);
  assert.match(minikubeCommand, /MINIKUBE_MEMORY_MB=8192/);
  assert.match(minikubeCommand, /MINIKUBE_K8S_VERSION='v1\.31\.0'/);

  const dockerContainersCommand = buildDockerContainersCommand();
  assert.match(dockerContainersCommand, /Docker is not installed\./);

  const nixSetupCommand = buildNixSetupCommand({
    installNix: true,
    enableFlakes: true,
    runGarbageCollect: true,
    packages: "git ripgrep,fd",
  });
  assert.match(nixSetupCommand, /NIX_INSTALL=1/);
  assert.match(nixSetupCommand, /NIX_ENABLE_FLAKES=1/);
  assert.match(nixSetupCommand, /NIX_RUN_GC=1/);
  assert.match(nixSetupCommand, /NIX_PACKAGES='git ripgrep fd'/);
  assert.match(nixSetupCommand, /apt-get install -y nix-bin/);
  assert.match(nixSetupCommand, /experimental-features = nix-command flakes/);
  assert.match(nixSetupCommand, /nix-collect-garbage -d/);

  const nixVersionCommand = buildNixVersionCommand();
  assert.match(nixVersionCommand, /nix --version/);
  assert.match(nixVersionCommand, /nix show-config/);

  const nixPackagesCommand = buildNixPackagesCommand();
  assert.match(nixPackagesCommand, /nix profile list --json/);
  assert.match(nixPackagesCommand, /nix-env -q --installed/);

  const nixStoreUsageCommand = buildNixStoreUsageCommand();
  assert.match(nixStoreUsageCommand, /nix-store --gc --print-live/);
  assert.match(nixStoreUsageCommand, /du -sh \/nix\/store/);

  const parsedNixJson = parseNixPackages(
    JSON.stringify({
      elements: [
        {
          attrPath: "legacyPackages.x86_64-linux.ripgrep",
          version: "14.1.0",
          originalUrl: "flake:nixpkgs",
        },
      ],
    })
  );
  assert.equal(parsedNixJson.length, 1);
  assert.equal(parsedNixJson[0].name, "ripgrep");
  assert.equal(parsedNixJson[0].version, "14.1.0");
  assert.equal(parsedNixJson[0].source, "flake:nixpkgs");

  const parsedNixLines = parseNixPackages("git-2.47.0\nhtop-3.3.0\ncustom-tool\n");
  assert.equal(parsedNixLines.length, 3);
  assert.equal(parsedNixLines[0].name, "git");
  assert.equal(parsedNixLines[0].version, "2.47.0");
  assert.equal(parsedNixLines[2].version, "—");

  const parsedMissingNix = parseNixPackages("nix is not installed.");
  assert.equal(parsedMissingNix.length, 0);
});
