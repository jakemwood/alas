# Frontend

This frontend is deployed to a static server (currently https://alas.krdf.org)

It was done this way so the frontend could be quickly loaded, and then only
minimal API payloads are shipped back and forth between the ALAS unit and
the browser.

## Deployment

Currently, deployment happens via git integration to Cloudflare pages.