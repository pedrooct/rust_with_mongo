
// conexão a mongoDB
conn = new Mongo("localhost:27017");
db = conn.getDB("hl");

db.createUser(
    {
        user: "docker",
        pwd: "docker",
        roles: [{ role: "dbOwner", db: "hl" }]
    }
)

//Apaga Todas as DB na mongo
var dbs = db.getMongo().getDBNames();
for (var i in dbs) {
    db = db.getMongo().getDB(dbs[i]);
    if (db.getName() != "admin" && db.getName() != "config" && db.getName() != "local" ) {
        print("dropping db " + db.getName());
        db.dropDatabase();
    }
}
//
db = db.getSiblingDB("hl"); // equivalent for "use <db>" command in mongo shell
db.createCollection("users");
db.createCollection("barricas"); // cria colecção 
db.createCollection("oleoes");



