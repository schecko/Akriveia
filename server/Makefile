
DEBUG_ARGS = --debug
RELEASE_ARGS = --release
FRONTEND_ARGS =  -p frontend --target wasm32-unknown-unknown
BACKEND_ARGS =  -p backend

all: build-back build-front

# build the backend, for independant work from the frontend
build-back:
	cargo build $(BACKEND_ARGS)

build-front:
	cargo web build $(FRONTEND_ARGS)

deploy: build-front build-back
	cargo web deploy -p frontend

# it looks like there is a bug with wasm right now, only frontend release works with yew fetch? https://github.com/DenisKolodin/yew/issues/455
run-back:
	cargo web deploy $(FRONTEND_ARGS) $(RELEASE_ARGS)
	cargo run $(BACKEND_ARGS)

# start the debug server, independant of the backend.
run-front:
	cargo web start $(FRONTEND_ARGS) $(RELEASE_ARGS) --auto-reload --host 0.0.0.0

