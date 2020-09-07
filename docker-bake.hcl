variable "TAG" {
	default = "latest"
}

variable "REGISTRY" {
	default = "docker.io"
}

variable "USERNAME" {
	default = "tkrs"
}

group "default" {
	targets = ["server", "reload"]
}

target "server" {
	dockerfile = docker/server/Dockerfile
	tags = ["${REGISTRY}/${USERNAME}/mmdb-server:${TAG}"]
}

target "reload" {
	dockerfile = docker/reload/Dockerfile
	tags = ["${REGISTRY}/{USERNAME}/mmdb-reload:${TAG}"]
}
