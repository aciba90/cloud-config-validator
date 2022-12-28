
```sh
ab -T application/json -p load-test/simple.json -c 1000 -n 20000 http://localhost:3000/v1/cloud-config/validate
```

Test machine: Ryzen 5 3600 + 16GiB RAM.
