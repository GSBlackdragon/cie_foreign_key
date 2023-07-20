# Comment utiliser le programme CIEForeignKey :
## 1) La mise en place :
Pour pouvoir utiliser le programme, plusieurs choses sont requises : 
- Une base de données SQL Server
- Une base de données postgres avec un dump de la V16 de Odoo (Le plus récent est le mieux, mais vous n'êtes pas obligé
de le refaire à chaque fois)
## 2) Utilisation :
La première utilité du programme est de **recréer les clés étrangères** dans la base de données SQLServer à partir des clés
présentes dans le dumb de la V16 d'Odoo. Pour ce faire, il vous suffit de rentrer les informations de connexion de vos bases de données dans le fichier de
configuration et d'**executer le programme** dans l'invité de commande : ```cie_foreign_key.exe```

Cependant, la création de ces clés engendre l'impossibilité de supprimer les tables, ce qui pose problème lorsque l'on
veut faire un import. De ce fait, le programme peut également **supprimer les clés étrangères** de la base de donnée
SQLServer. Pour ce faire, **executez le programme** en ajoutant le **paramètre -c** : ```cie_foreign_key.exe -c```
## 3) Le fichier de configuration :
```
sql_server_host = '' 
sql_server_port = 
sql_server_user = ''
sql_server_pass = ''
sql_server_db = '' 
postgres_host = ''
postgres_port = ''
postgres_user = ''
postgres_pass = ''
postgres_db = ''
```
*Sauf exeption, toutes les valeurs demandées sont des string*

***sql_server_host*** est l'adresse du server SQLServer.

***sql_server_port*** est le port sur lequel le server SQLServer écoute. **ATTENTION** contrairement aux autres valeur, 
on demande ici un **entier** et pas un string !

***sql_server_user*** est le nom de l'utilisateur SQLServer.

***sql_server_pass*** est le mot de pass de l'utilisateur choisi.

***sql_server_db*** est le nom de la base de données utilisée dans SQLServer.

***postgres_host*** est l'adresse du server postgres nécessaire pour récuperer les clés étrangères.

***postgres_port*** est le port sur lequel le serveur postgres écoute.

***postgres_user*** est le nom de l'utilisateur postgres.

***postgres_pass*** est le mot de pass de l'utilisateur choisi.

***postgres_db*** est le nom de la base de données contenant le dump de la V16.
## 4) Informations complémentaires :
- Le fichier de configuration doit ce trouver dans le même dossier que le programme !
- Le dossier des logs peut être supprimer sans problème, il sera automatiquement recréé.