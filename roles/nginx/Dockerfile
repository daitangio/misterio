
# Nginx 118
# FROM nginxinc/nginx-unprivileged:1.21.1-alpine
FROM nginx:1.18.0-alpine-perl
# FROM nginx:1.21.6-alpine
EXPOSE 8080
COPY nginx.conf /etc/nginx/nginx.conf
COPY default.conf /etc/nginx/conf.d/default.conf

#USER 1001
CMD [ "nginx", "-g", "daemon off;"]