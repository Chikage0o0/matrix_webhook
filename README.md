# Matrix-Webhook

## https://github.com/Chikage0o0/matrix-bot
## 介绍
通过Matrix SDK实现一个send接口，用于发送消息到Matrix中的房间
接口格式为:

POST http://{host}:{port}/send
```json
{
    "msg": "test",
}
```
如果在环境变量中设置了`TOKEN`,则需要在请求头中添加`Authorization: Bearer {TOKEN}`

## E2EE
当申请验证后，会在终端输出相应引导，按照提示操作即可。

## 使用
#### Docker
```shell
docker run -d --name matrix_webhook      \
    -e HOME_SERVER_URL="https://xxx.xxx" \
    -e ROOM_ID='!PWPurdafsdfasd:xx.xxx'  \
    -e USER="x"                          \
    -e PASSWORD="x"                      \
    -e TOKEN="x"                         \
    -e LISTEN='0.0.0.0'                  \
    -p 8080:3000                         \
    -v ./matrix_webhook:/matrix_webhook  \
    --restart unless-stopped chikage/matrix_webhook:latest
```
