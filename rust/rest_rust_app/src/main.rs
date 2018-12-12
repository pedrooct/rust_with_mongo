#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
#[macro_use(bson, doc)]extern crate bson;
extern crate yyid;
extern crate serde;
extern crate serde_json;
extern crate mongodb;
extern crate pwhash;
extern crate rocket_cors;

use rocket::{get, routes};
use rocket::http::Method;
use rocket_cors::{AllowedOrigins, AllowedHeaders};
use bson::Document;
use pwhash::bcrypt;
use mongodb::Bson;
use bson::oid::ObjectId;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use rocket_contrib::json::{Json,JsonValue};
use yyid::yyid_string;

#[derive(Serialize, Deserialize, Debug,Default)]
struct User{
    email: String,
    password: String,
    access_token: String
}
// default permite iniciar a estrutura a 0
#[derive(Serialize, Deserialize, Debug,Default)]
struct Barrica { 
    idb: String,// ID = _id que é gerado pela BD , ID só usado quando recebemos da BD
    nome : String,
    morada: String,
    cidade: String,
    peso_barrica: f64, // float peso da barrica sem conteudo
    peso_maximo: f64, // peso maximo que esta consegue suportar de conteudo
    peso_atual: f64, // peso atual de conteudo 
}
#[derive(Serialize, Deserialize,Debug,Default)]
struct Oleoes{
    ido: String, // ID só usado quando recebemos da BD
    nome : String,
    morada: String,
    cidade: String,
    peso_oleao: f64,
    peso_maximo: f64,
    peso_atual: f64,
    garrafas_inseridas: u64, // numero unsigned 64 bits
}


// função para retornar um cliente para a conexão
fn get_mongo_client()-> std::sync::Arc<mongodb::ClientInner> {
    let client =Client::connect("mongodb", 27017).ok().expect("Failed to initialize client.");
    let db = client.db("hl");
    let _auth_result = db.auth("docker", "docker");
    client
}

// funções teste para mudar a colecção
fn define_collection_barricas(client : Client)-> mongodb::coll::Collection{
    client.db("hl").collection("barricas")
}
fn define_collection_oleoes(client : Client)-> mongodb::coll::Collection{
    client.db("hl").collection("oleos")
}
fn define_collection_users(client : Client)-> mongodb::coll::Collection{
    client.db("hl").collection("users")
}
// Permite gerar um numero de 128 bits aleatório para servir de token
fn generate_token()-> String{
    yyid_string()
}

fn verify_exist_user(email: &String)-> bool{
    let client = get_mongo_client();
    let coll = define_collection_users(client);
    let doc = doc!{
        "email" => email.to_string(),
    };
    let cursor = coll.find(Some(doc.clone()), None).unwrap();
    for result in cursor {
        if let Ok(item) = result{
             if let Some(&Bson::String(ref _email_aux)) = item.get("email"){
                return false; // quando dentro de ciclos deve-se indicar claramente que é para existir um return           
            }
        }
    }
    true
}
fn verify_token(token : String)-> bool{
    let client = get_mongo_client();
    let coll = define_collection_users(client);
    let doc = doc!{
        "access_token" => token.to_string(),
    };
    let cursor = coll.find(Some(doc.clone()), None).unwrap();
    for result in cursor {
        if let Ok(item) = result{
            if let Some(&Bson::String(ref _email_aux)) = item.get("email"){
                return true;
            }
        }
    }
    false
}


#[post("/login", format = "application/json", data ="<user>")]
fn login(user : Json<User>)->JsonValue{
    let client = get_mongo_client();
    let coll = define_collection_users(client);
    let doc = doc!{
        "email" => user.email.to_string(),
    };
    let cursor = coll.find(Some(doc.clone()), None).unwrap();
    for result in cursor {
        if let Ok(item) = result{
            if let Some(&Bson::String(ref access_token)) = item.get("access_token"){
                if let Some(&Bson::String(ref password_aux)) = item.get("password"){
                    if bcrypt::verify(user.password.to_string(), password_aux){
                        return json!({"token":access_token.to_string()});
                    }
                    
                }
            }
        }
    }
    json!({"status":"error"})
}
#[post("/register",format = "application/json",data = "<user>")]
fn register_user(user: Json<User>)-> JsonValue{
    if verify_exist_user(&user.email)
    {
        let client = get_mongo_client();
        let coll = define_collection_users(client);
        let access_token= generate_token();
        let passcrypt = bcrypt::hash(user.password.to_string()).unwrap();
        println!("{}",access_token.to_string());
        match coll.insert_one(doc!{ 
            "email" => user.email.to_string(),
            "password"=> passcrypt,
            "access_token"=> access_token.to_string()
        }, None) {
            Ok(_) => json!({"token":access_token.to_string()}),
            Err(_e) => json!({"status":"error"})
        }
    }
    else{
        json!({"status":"error"})
    }
    
}

//Função para o inserir
fn save_in_mongo_barrica(nome: &String,morada: &String,cidade: &String, peso_barrica: f64,peso_maximo:f64,peso_atual:f64)-> bool{
    let client = get_mongo_client();
    let col = define_collection_barricas(client);
    match col.insert_one(doc!{ 
        "nome" => nome,
        "morada"=> morada,
        "cidade"=> cidade,
        "peso_barrica" => peso_barrica,
        "peso_maximo"=> peso_maximo,
        "peso_atual"=> peso_atual
    }, None) {
        Ok(_) => true ,
        Err(_e) => false
    } 
}
//Guardar oleoes na mongo
fn save_in_mongo_oleoes(nome: &String,morada: &String,cidade: &String, garrafas_inseridas: u64,peso_oleao: f64,peso_maximo: f64,peso_atual:f64)-> bool{
    let client = get_mongo_client();
    let col = define_collection_oleoes(client);
    match col.insert_one(doc!{ 
        "nome" => nome,
        "morada"=> morada,
        "cidade"=> cidade,
        "peso_oleao" => peso_oleao,
        "peso_maximo"=> peso_maximo,
        "peso_atual"=> peso_atual,
        "garrafas_inseridas"=> garrafas_inseridas
    }, None) {
        Ok(_) => true ,
        Err(_e) => false
    } 
}

//Permite inserir um elemento novo na colecção de barricas
#[post("/<token>/insert",format = "application/json", data = "<oleos>")]
fn insert_oleos(token: String,oleos: Json<Oleoes>) -> JsonValue {
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    if save_in_mongo_oleoes(&oleos.nome,&oleos.morada,&oleos.cidade,oleos.garrafas_inseridas,oleos.peso_oleao,oleos.peso_maximo,oleos.peso_atual) {
        json!({ "status": "Done!" })
    }
    else{
       json!({ "status": "error" })
    }
}


//Permite inserir um elemento novo na colecção de barricas
#[post("/<token>/insert",format = "application/json", data = "<barrica>")]
fn insert_barricas(token : String,barrica: Json<Barrica>) -> JsonValue {
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    if save_in_mongo_barrica(&barrica.nome,&barrica.morada,&barrica.cidade,barrica.peso_barrica,barrica.peso_maximo,barrica.peso_atual) {
        json!({ "status": "Done!" })
    }
    else{
       json!({ "status": "error" })
    }
}
//Função permite editar barrica
#[put("/<token>/editar", format="application/json", data ="<barrica>")]
fn edit_barricas(token: String,barrica: Json<Barrica>)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let client = get_mongo_client();
    let coll = define_collection_barricas(client);
    let filter = doc!{"_id" => Bson::ObjectId(ObjectId::with_string(&barrica.idb).unwrap())};
    let update = doc!{"$set" =>{"nome"=>&barrica.nome.to_string(),
        "morada"=> &barrica.morada.to_string(),
        "cidade" => &barrica.cidade.to_string(),
        "peso_barrica"=> barrica.peso_barrica,
        "peso_maximo"=> barrica.peso_maximo,
        "peso_atual"=> barrica.peso_atual}};
    match coll.update_one(filter, update, None){
        Ok(_)=> json!({ "status": "Done!" }),
        Err(_)=> json!({ "status": "Erro!" })
    }
}

//Função permite editar oleo
#[put("/<token>/editar", format="application/json", data ="<oleao>")]
fn edit_oleao(token: String,oleao: Json<Oleoes>)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let client = get_mongo_client();
    let coll = define_collection_oleoes(client);
    let filter = doc!{"_id" => Bson::ObjectId(ObjectId::with_string(&oleao.ido).unwrap())};
    let update = doc!{"$set" =>{"nome"=>&oleao.nome.to_string(),
        "morada"=> &oleao.morada.to_string(),
        "cidade" => &oleao.cidade.to_string(),
        "peso_oleao"=> oleao.peso_oleao,
        "peso_maximo"=> oleao.peso_maximo,
        "peso_atual"=> oleao.peso_atual}};
    match coll.update_one(filter, update, None){
        Ok(_)=> json!({ "status": "Done!" }),
        Err(_)=> json!({ "status": "Erro!" })
    }
}
#[put("/<token>/incrementgarrafas/<currval>/<id>")]
fn increment_garrafas(token: String,currval: u64,id : String)-> JsonValue{
     if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let client = get_mongo_client();
    let coll = define_collection_oleoes(client);
    let filter = doc!{"_id" => Bson::ObjectId(ObjectId::with_string(&id).unwrap())};
    let update = doc!{"$set" =>{"garrafas_inseridas"=>currval}};
    match coll.update_one(filter, update, None){
        Ok(_)=> json!({ "status": "Done!" }),
        Err(_)=> json!({ "status": "Erro!" })
    }
}
#[put("/<token>/resetgarrafas/<id>")]
fn reset_garrafas(token: String,id : String)-> JsonValue{
     if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let client = get_mongo_client();
    let coll = define_collection_oleoes(client);
    let filter = doc!{"_id" => Bson::ObjectId(ObjectId::with_string(&id).unwrap())};
    let update = doc!{"$set" =>{"garrafas_inseridas"=>0}};
    match coll.update_one(filter, update, None){
        Ok(_)=> json!({ "status": "Done!" }),
        Err(_)=> json!({ "status": "Erro!" })
    }
}
//Permite apagar um elemento com base no ID na coleção barricas
#[delete("/<token>/delete/<id>")]
fn delete_barrica(token: String,id: String)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let client = get_mongo_client();
    let coll = define_collection_barricas(client);
   match coll.delete_one(doc!{"_id" => Bson::ObjectId(ObjectId::with_string(&id).unwrap())}, None){
       Ok(_)=> json!({ "status": "Done!" }),
       Err(_)=> json!({ "status": "Erro!" })
   }
}
//Permite apagar um elemento com base no ID na colecção oleoes
#[delete("/<token>/delete/<id>")]
fn delete_oleao(token: String,id: String)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let client = get_mongo_client();
    let coll = define_collection_oleoes(client);
   match coll.delete_one(doc!{"_id" => Bson::ObjectId(ObjectId::with_string(&id).unwrap())}, None){
       Ok(_)=> json!({ "status": "Done!" }),
       Err(_)=> json!({ "status": "Erro!" })
   }
}

// função de prcoura de barricas devolve toda a informação
fn search_mongo_barricas(doc:Document)-> Vec<Barrica> {
    let client = get_mongo_client();
    let coll = define_collection_barricas(client);
    let cursor = coll.find(Some(doc.clone()), None).unwrap();
    let mut v: Vec<Barrica> = Vec::new();
    for result in cursor {
        if let Ok(item) = result{
             if let Some(&Bson::String(ref morada_aux)) = item.get("morada"){
                if let Some(&Bson::FloatingPoint(ref peso_barrica_aux)) = item.get("peso_barrica"){
                    if let Some(&Bson::FloatingPoint(ref peso_maximo_aux)) = item.get("peso_maximo"){
                        if let Some(&Bson::FloatingPoint(ref peso_atual_aux)) = item.get("peso_atual"){
                            if let Some(&Bson::ObjectId(ref idb_aux)) = item.get("_id"){                    
                                if let Some(&Bson::String(ref cidade_aux)) = item.get("cidade"){ 
                                    if let Some(&Bson::String(ref nome_aux)) = item.get("nome"){
                                        let barrica= Barrica{
                                            idb:idb_aux.to_string(),
                                            nome:nome_aux.to_string(),
                                            cidade:cidade_aux.to_string(),
                                            morada:morada_aux.to_string(),
                                            peso_barrica:*peso_barrica_aux,
                                            peso_atual:*peso_atual_aux,
                                            peso_maximo:*peso_maximo_aux};
                                        v.push(barrica);
                                    }
                                } 
                            }
                        }
                    }  
                }
            }  
        }
    }
    v
}
// funcao de procura na mongo ... devolve toda a informacao sobre oleoes
fn search_mongo_oleoes(doc:Document)-> Vec<Oleoes>{
    let client = get_mongo_client();
    let coll = define_collection_oleoes(client);
    let cursor = coll.find(Some(doc.clone()), None).unwrap();
    let mut v: Vec<Oleoes> = Vec::new(); // Vector para armazenar o dados
    for result in cursor {
        if let Ok(item) = result{
            if let Some(&Bson::String(ref morada_aux)) = item.get("morada"){
                if let Some(&Bson::FloatingPoint(ref peso_oleao_aux)) = item.get("peso_oleao"){
                    if let Some(&Bson::FloatingPoint(ref peso_maximo_aux)) = item.get("peso_maximo"){
                        if let Some(&Bson::FloatingPoint(ref peso_atual_aux)) = item.get("peso_atual"){
                            if let Some(&Bson::ObjectId(ref idb_aux)) = item.get("_id"){                    
                                if let Some(&Bson::String(ref cidade_aux)) = item.get("cidade"){ 
                                    if let Some(&Bson::String(ref nome_aux)) = item.get("nome"){
                                        if let Some(&Bson::I64(ref garrafas_inseridas_aux)) = item.get("garrafas_inseridas"){
                                            let oleao= Oleoes{
                                                ido:idb_aux.to_string(), // ID só usado quando recebemos da BD
                                                nome:nome_aux.to_string(),
                                                morada:morada_aux.to_string(),
                                                cidade:cidade_aux.to_string(),
                                                peso_oleao:*peso_oleao_aux,
                                                peso_atual:*peso_atual_aux,
                                                peso_maximo:*peso_maximo_aux,
                                                garrafas_inseridas:*garrafas_inseridas_aux as u64};
                                            v.push(oleao);
                                        }
                                    }
                                }
                            }
                        } 
                    }
                }  
            }  
        }
    }
    v
}
//Procura de elementos na BD por nome
#[get("/<token>/printbyname/<name>")]
fn print_collection_barricas_byname(token: String,name: String)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    json!(search_mongo_barricas(doc!{"nome" => name.to_string()}))
   
}

//Procura na DB por o menor peso
#[get("/<token>/printbysmallerwieght/<peso>")]
fn print_collection_barricas_smaller_weight(token: String,peso: f64)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let mut v: Vec<Barrica> = Vec::new();
    let vec_aux = search_mongo_barricas(doc!{});
    for x in &vec_aux {
        if x.peso_atual < peso{
            let barrica= Barrica{
                idb:x.idb.to_string(),
                nome: x.nome.to_string(),
                cidade:x.cidade.to_string(),
                morada: x.morada.to_string(),
                peso_barrica:x.peso_barrica,
                peso_atual: x.peso_atual,
                peso_maximo:x.peso_maximo};
            v.push(barrica);
        }
    }
    json!(v)
}
//Procura na DB por o maior peso
#[get("/<token>/printbybiggerwieght/<peso>")]
fn print_collection_barricas_bigger_weight(token: String,peso: f64)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let mut v: Vec<Barrica> = Vec::new();
    let vec_aux = search_mongo_barricas(doc!{});
    for x in &vec_aux {
        if x.peso_atual > peso{
            let barrica= Barrica{
                idb:x.idb.to_string(),
                nome: x.nome.to_string(),
                cidade:x.cidade.to_string(),
                morada: x.morada.to_string(),
                peso_barrica:x.peso_barrica,
                peso_atual: x.peso_atual,
                peso_maximo:x.peso_maximo};
            v.push(barrica);
        }
    }
    json!(v)           
}
//Procura na DB por o menor peso
#[get("/<token>/printbybetweenwieght/<pesosmall>/<pesobigger>")]
fn print_collection_barricas_between_weight(token: String,pesosmall: f64,pesobigger:f64)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let mut v: Vec<Barrica> = Vec::new();
    let vec_aux = search_mongo_barricas(doc!{});
    for x in &vec_aux {
        if x.peso_atual > pesosmall && x.peso_atual < pesobigger{
            let barrica= Barrica{
                idb:x.idb.to_string(),
                nome: x.nome.to_string(),
                cidade:x.cidade.to_string(),
                morada: x.morada.to_string(),
                peso_barrica:x.peso_barrica,
                peso_atual: x.peso_atual,
                peso_maximo:x.peso_maximo};
            v.push(barrica);
        }
    }
    json!(v)
}

//Procura de elementos na BD por ID
#[get("/<token>/printbyid/<id>")]
fn print_collection_barricas_byid(token: String,id: String)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    json!(search_mongo_barricas(doc!{ "_id" => Bson::ObjectId(ObjectId::with_string(&id).unwrap())}))
}
//Procura na DB por o menor peso
#[get("/<token>/printbycidade/<cidade>")]
fn print_collection_barricas_bycidade(token: String,cidade: String)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    json!(search_mongo_barricas(doc!{"cidade" => cidade.to_string()}))
   
}

// imprime todas as barricas !
#[get("/<token>/printall")]
fn print_collection_barricas_all(token: String)-> JsonValue{
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    json!(search_mongo_barricas(doc!{}))
}

#[get("/<token>/getpercentageweight/<id>")]
fn get_percentage_weight(token : String, id: String)-> JsonValue {
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    let vec_aux = search_mongo_barricas(doc!{ "_id" => Bson::ObjectId(ObjectId::with_string(&id).unwrap())});
    if vec_aux.is_empty(){
       return json!({"status": "no data"});
    }
    let mut aux = vec_aux[0].peso_atual * 100.0;
    aux = aux / vec_aux[0].peso_maximo;
    json!({"percentage":aux})
}


// pesquisas de oleos !

#[get("/<token>/printbyid/<id>")]
fn print_collection_oleoes_byid(token: String, id : String)-> JsonValue{
     if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    json!(search_mongo_oleoes(doc!{ "_id" => Bson::ObjectId(ObjectId::with_string(&id).unwrap())}))
}
#[get("/<token>/printbyname/<name>")]
fn print_collection_oleoes_byname(token: String, name : String)-> JsonValue{
     if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    json!(search_mongo_oleoes(doc!{ "nome" => name.to_string()}))
}

// imprime todas os oleoes !
#[get("/<token>/printall")]
fn print_collection_oleoes_all(token: String)-> JsonValue{
    println!("{}",token);
    if !verify_token(token) {
        return json!({"status":"token not valid"});
    }
    json!(search_mongo_oleoes(doc!{}))
}

#[get("/")]
fn index() -> &'static str {
    "It's live and working!!"
}


fn main() {
    // CORS !
    //
    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["http://localhost:4200"]);
    assert!(failed_origins.is_empty());
    let options = rocket_cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get,Method::Post,Method::Put,Method::Delete].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Content-Type", "Bearer", "Bearer ", "content-type", "Origin", "Accept"]),
        ..Default::default()
    };
    // _ permite ignorar o facto de não usarmos a variavel
    
    rocket::ignite()
    .mount("/home/barricas", routes![
        print_collection_barricas_bycidade,
        print_collection_barricas_byname,
        print_collection_barricas_byid,
        print_collection_barricas_all,
        print_collection_barricas_bigger_weight,
        print_collection_barricas_smaller_weight,
        print_collection_barricas_between_weight,
        get_percentage_weight,
        insert_barricas,
        delete_barrica,
        edit_barricas])
    .mount("/home/oleoes",routes![
        insert_oleos,
        delete_oleao,
        edit_oleao,
        reset_garrafas,
        increment_garrafas,
        print_collection_oleoes_all,
        print_collection_oleoes_byid,
        print_collection_oleoes_byname])
    .mount("/", routes![
        index,
        register_user,
        login
    ]).attach(options).launch();
}
