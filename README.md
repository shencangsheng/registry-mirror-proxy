# registry-mirror-proxy

[![Docker Pulls](https://img.shields.io/docker/pulls/shencangsheng/registry-mirror-proxy.svg)](https://hub.docker.com/r/shencangsheng/registry-mirror-proxy)

English | [简体中文](./i18n/README.zh-cn.md)

Proxy Registry API

## Features:

1. Intercept the Get Docker image API, synchronize the image to the Docker registry, and then forward the request to the registry server.

## Principle

```mermaid
graph TD;
    A[Docker request] --> B[Docker registry proxy];
    B --> C{Get Docker image API?};
    C -- Yes --> D[Docker pull image];
    C -- No --> E[Docker registry server];
    D --> F[Upload Docker registry];
    F --> E
    E -- Response --> B
    B -- Response --> A
```

## Credits

This project was inspired by the [shencangsheng/easy-registry-mirror](https://github.com/shencangsheng/easy-registry-mirror) available in the GitHub project.

## License

A short snippet describing the license (MIT)

MIT © Cangsheng Shen