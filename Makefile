build-frontend:
	cd frontend
	npm run build
	rsync -r dist/ ridgeline@ridgeline-pi:/home/ridgeline/static
