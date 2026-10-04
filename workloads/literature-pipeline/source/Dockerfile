FROM alpine:3.19

# Instalar dependencias
RUN apk add --no-cache \
    bash \
    gawk \
    graphviz \
    git \
    make

# Crear directorio de trabajo
WORKDIR /app

# Copiar el pipeline
COPY . /app/

# Hacer ejecutables
RUN chmod +x /app/pipeline.sh /app/scripts/*.awk /app/tests/*.sh

# Punto de entrada
ENTRYPOINT ["/app/pipeline.sh"]
CMD ["--help"]
