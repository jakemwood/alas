set dotenv-load

[working-directory('frontend')]
@build-frontend:
    npm run build

[working-directory('frontend')]
@deploy-frontend:
    rsync -r dist/ ${PI_USERNAME}@${PI_HOSTNAME}:/home/${PI_USERNAME}/static

build:
    cross build --target aarch64-unknown-linux-gnu --release

deploy:
    scp target/aarch64-unknown-linux-gnu/release/web ${PI_USERNAME}@${PI_HOSTNAME}:/home/${PI_USERNAME}
