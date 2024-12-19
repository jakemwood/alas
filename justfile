[working-directory('frontend')]
@build-frontend:
    npm run build

build:
    cross build --target aarch64-unknown-linux-gnu --release

deploy:
    scp target/aarch64-unknown-linux-gnu/release/web ridgeline@ridgeline-live.local:/home/ridgeline
