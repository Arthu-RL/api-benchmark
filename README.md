# api-benchmark

A ideia é fazer vários posts enviando um body qualquer e avaliar várias métricas de performace da API que recebe estas requests

## Notes:

`file_path_for_body_data` seria um arquivo que seu conteúdo vai servir de body das requests

## Use

Desafio: Precisa de um POST implementado na API path/apibenchmark, que retorne um body qualquer recebido

```bash
target/debug/apibenchmark --url "http://localhost:{port}/apibenchmark" --pool-max-idle-per-host 100 --concurrency 100 --requests-per-worker 1000 --file-path-for-body-data "image.png"
```
