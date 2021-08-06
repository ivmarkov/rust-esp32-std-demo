.PHONY: build
build:
	docker run --rm -v $(PWD):/app -w /app esp32-std-build -- cargo build

.PHONY: release
release:
	docker run --rm -v $(PWD):/app -w /app esp32-std-build -- cargo build --release

.PHONY: build-docker-image
build-docker-image:
	docker build . -t esp32-std-build
