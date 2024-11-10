## API for Simple image upload and processing

- Image uploading
- Connect to Cloud Bucket or Minio. Supports S3 interface
- Image downloading
- Image resizing: Default sizes and resize on demand
- Bearer Auth on image request


## Instructions

- Set up the env variables but copying the .env.example file
- run `docker-compose up --build`

## Usage
Post an image to the /upload folder

```
curl -X POST \
  -H "Content-Type: multipart/form-data" \
  -F "image=@/path/to/your/image.jpg" \
  http://your-server/upload
  
```