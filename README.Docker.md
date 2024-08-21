### Building and running your application

When you're ready, start your application by running:
`docker compose up --build`.

Your application will be available at http://localhost:9092.

### Deploying your application to the cloud

First, build your image, e.g.: `docker build -t myapp .`.
If your cloud uses a different CPU architecture than your development
machine (e.g., you are on a Mac M1 and your cloud provider is amd64),
you'll want to build the image for that platform, e.g.:
`docker build --platform=linux/amd64 -t myapp .`.

Then, push it to your registry, e.g. `docker push myregistry.com/myapp`.

Consult Docker's [getting started](https://docs.docker.com/go/get-started-sharing/)
docs for more detail on building and pushing.

### References
* [Docker's Rust guide](https://docs.docker.com/language/rust/)


## Building app with SQLX
In order to build application with sqlx, the macros used in code need to validate SQL
Tryding to use the updated query-cache from development
Set
```
ENV SQLX_OFFLINE true
```
Mount generated sqlx cache to build
```
--mount=type=bind,source=.sqlx,target=.sqlx \
```

```
TAG="0.21" 
docker build -q -t "arnecdn/greeting-processor-rust:${TAG}" . &&
mkdir -p .docker && docker image save -o .docker/greeting-processor-rust.tar "arnecdn/greeting-processor-rust:${TAG}" &&
minikube image load .docker/greeting-processor-rust.tar
```


For installation of Keda, use Helm with the following commandline parameters. 
Setting namespace=default because of some unsolved problem with accessing Kafka in different namespace

```
helm install keda kedacore/keda --namespace default
```

