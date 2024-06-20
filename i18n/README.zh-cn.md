# registry-mirror-proxy

[![Docker Pulls](https://img.shields.io/docker/pulls/shencangsheng/registry-mirror-proxy.svg)](https://hub.docker.com/r/shencangsheng/registry-mirror-proxy)

[English](https://github.com/shencangsheng/registry-mirror-proxy) | 简体中文

代理自建 Registry API

## Features:

1. 拦截 Docker Pull Image 请求，将 Image 同步到 Docker Registry，再返回 Image

## Upcoming Features

1. npm Registry API

## Principle

```mermaid
graph TD;
    A[Docker Request] --> B[Docker Registry Proxy];
    B --> C{docker pull?};
    C -- Yes --> D[docker pull image];
    C -- No --> E[Docker Registry Server];
    D --> F[Upload Docker Registry];
    F --> E
    E -- Response --> B
    B -- Response --> A
```

## Credits

This project was inspired by the [shencangsheng/easy-registry-mirror](https://github.com/shencangsheng/easy-registry-mirror) available in the GitHub project.

## License

A short snippet describing the license (MIT)

MIT © Cangsheng Shen