IMAGE_NAME=rust_runtime
CONTAINER_NAME=rust_runtime

build:
	docker build \
		-f docker/Dockerfile \
		-t ${IMAGE_NAME} \
		--force-rm \
		.

run:
	docker run \
		-dit \
		-v ${PWD}:/workspace \
		--name ${CONTAINER_NAME} \
		--privileged \
		${IMAGE_NAME}

exec:
	docker exec \
		-it \
		${CONTAINER_NAME} \
		bash

start:
	docker start ${CONTAINER_NAME}
	
stop:
	docker stop ${CONTAINER_NAME}
