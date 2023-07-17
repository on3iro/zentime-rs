# Lists all just tasks in order of their appearance inside the justfile
default:
    just --list --unsorted

# Builds and starts a debian container in interactive mode with the repository mounted inside
env-linux:
    docker build -f docker/linux/Dockerfile -t zentime-linux .
    docker run --volume $(pwd):/zentime -it zentime-linux bash

# Shows outdated direct dependencies
outdated:
    cargo outdated --color "Always" -d 1
