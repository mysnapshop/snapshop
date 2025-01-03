# !/bin/sh
sudo rm -rf /srv/mongodb
sudo mkdir -p /srv/mongodb/rs0-0  /srv/mongodb/rs0-1 /srv/mongodb/rs0-2
sudo chown $USER:$USER -R /srv/mongodb/
mongod --replSet rs0 --port 27017 --bind_ip localhost --dbpath /srv/mongodb/rs0-0 --oplogSize 128 &\
mongod --replSet rs0 --port 27018 --bind_ip localhost --dbpath /srv/mongodb/rs0-1 --oplogSize 128 &\
mongod --replSet rs0 --port 27019 --bind_ip localhost --dbpath /srv/mongodb/rs0-2 --oplogSize 128

# rsconf = {
#   _id: "rs0",
#   members: [
#     {
#      _id: 0,
#      host: "localhost:27017"
#     },
#     {
#      _id: 1,
#      host: "localhost:27018"
#     },
#     {
#      _id: 2,
#      host: "localhost:27019"
#     }
#    ]
# }