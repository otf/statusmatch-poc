.PHONY: deploy
deploy:
	cp -r ../frontend/public ./
	git add public/ && git commit -m 'add public to deploy static.'
	-@cargo shuttle deploy
	git reset --hard HEAD^