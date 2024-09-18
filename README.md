# NOT MAINTAINED
We ended up using other architecture, so this repo is what used to be our backend. not anymore

## backend
Art gallery companion app's backend made by Artizans

## start up

1. create .env first. refer to .env.example file as an example
2. then run following commands

```Shell
docker compose up --build -d
docker compose logs -f
```

## clean up when finished

```Shell
docker compose down
```

## What we did to manually push container to ECR

only first try.

```Shell
aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT_ID.dkr.ecr.ap-northeast-1.amazonaws.com
```

```Shell
docker buildx build --platform linux/amd64 -t artizans_webserver .

docker images # check IMAGE ID of image you built before

docker tag IMAGE_ID $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com/artizans/webserver

docker push $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com/artizans/webserver
```

Also we created new security group that allow all inbound access, and attach it to fargate. like 0.0.0.0/0 thing
