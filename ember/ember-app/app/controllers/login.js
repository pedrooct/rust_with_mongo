import Controller from '@ember/controller';

export default Controller.extend({
    actions: {
        login: function () {
            var email = this.get('email');
            var password = this.get('password');
            if (!email || !password) {
                this.set("showErrors", true);
            }
            var user = new Object();
            user.email = email;
            user.password = password;
            user.access_token = "0";
            var UserJson = JSON.stringify(user);
            fetch('http://localhost:8008/login', {
                headers: {
                    "content-type": "application/json"
                },
                method: 'post',
                body: UserJson
            }).then(res => res.json())
                .then(function (response) {
                    if (response.status == "error") {
                        alert("Email ou palavra-passe inválidos");
                    }
                    if (response.token) {
                        sessionStorage.setItem("token", response.token);
                        
                    }

                })
                .catch(function (error) {
                    alert("Algo correu mal com o pedido ! tente mais tarde: " + error);
                })
        }
    }
});
