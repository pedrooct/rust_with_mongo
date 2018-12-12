FROM node:8.12-alpine


#Copia para o WORKDIR a aplicação angular !
WORKDIR /home/ember/app

#install dependencies
RUN npm install -g ember-cli

COPY ./ember/ember-app/ .

#Garante que a ultima alteração será sempre atualiza em real-time
#CMD ng serve --host 0.0.0.0
#Resolve problema de indexação dos NODE_modules , e permite segurança na aplicação
CMD npm install && ember serve