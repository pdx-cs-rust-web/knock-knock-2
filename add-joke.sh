#!/bin/sh
PW=`cat secrets/reg_password.txt`
CREDS="{
  \"email\": \"bart.massey@gmail.com\",
  \"full_name\": \"Bart Massey\",
  \"password\": \"$PW\"
}"

ACCESS_TOKEN=`curl -s -X POST -H "Content-type: application/json" \
     -d "$CREDS" \
     http://localhost:3000/api/v1/register | jq .access_token | sed 's/"//g'`

JOKE='{
  "answer_who": "Nugent, nujoke.",
  "id": "nugent",
  "source": "Bart Massey",
  "tags": [
    "original", "bad"
  ],
  "whos_there": "Nugent"
}'

curl -X POST -H "Content-type: application/json"  \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     -d "$JOKE" http://localhost:3000/api/v1/add-joke
