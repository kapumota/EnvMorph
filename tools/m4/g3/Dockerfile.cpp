FROM envmorph-g3-inih-offline:local

USER root

# Reproducción del cambio de ingeniería que incorpora C++.
# Esta construcción usa la red para apt-get; los experimentos se ejecutan sin red.
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y \
       --no-install-recommends g++ \
    && command -v g++ \
    && command -v c++ \
    && rm -rf /var/lib/apt/lists/*
