env-linux:
    docker build -f docker/linux/Dockerfile -t zentime-linux .
    docker run --volume $(pwd):/zentime -it zentime-linux bash

