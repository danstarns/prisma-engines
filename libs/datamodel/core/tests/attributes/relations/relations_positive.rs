use crate::{common::*, with_header, Provider};
use datamodel::render_datamodel_to_string;

#[test]
fn must_add_referenced_fields_on_both_sides_for_many_to_many_relations() {
    let dml = indoc! {r#"
        model User {
          user_id Int    @id
          posts   Post[]
        }

        model Post {
          post_id Int    @id
          users   User[]
        }
    "#};

    let schema = parse(dml);

    schema
        .assert_has_model("User")
        .assert_has_relation_field("posts")
        .assert_relation_referenced_fields(&["post_id"]);
    schema
        .assert_has_model("Post")
        .assert_has_relation_field("users")
        .assert_relation_referenced_fields(&["user_id"]);
}

#[test]
fn settings_must_be_deteced() {
    let dml = indoc! {r#"
        model Todo {
          id       Int  @id
          parentId Int?

          child_todos Todo[] @relation("MyRelation")
          parent_todo Todo? @relation("MyRelation", fields: parentId, references: id)
        }
    "#};

    let schema = parse(dml);

    schema
        .assert_has_model("Todo")
        .assert_has_relation_field("parent_todo")
        .assert_relation_to("Todo")
        .assert_relation_referenced_fields(&["id"])
        .assert_arity(&dml::FieldArity::Optional);
}

#[test]
fn resolve_relation() {
    let dml = indoc! {r#"
        model User {
          id Int @id
          firstName String
          posts Post[]
        }

        model Post {
          id     Int    @id
          text   String
          userId Int

          user User @relation(fields: [userId], references: [id])
        }
    "#};

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model
        .assert_has_scalar_field("firstName")
        .assert_base_type(&ScalarType::String);
    user_model
        .assert_has_relation_field("posts")
        .assert_relation_to("Post")
        .assert_arity(&dml::FieldArity::List);

    let post_model = schema.assert_has_model("Post");
    post_model
        .assert_has_scalar_field("text")
        .assert_base_type(&ScalarType::String);
    post_model.assert_has_relation_field("user").assert_relation_to("User");
}

#[test]
fn resolve_related_field() {
    let dml = indoc! {r#"
        model User {
          id        Int    @id
          firstName String @unique
          posts Post[]
        }

        model Post {
          id            Int    @id
          text          String
          userFirstName String
          user          User   @relation(fields: [userFirstName], references: [firstName])
        }
    "#};

    let schema = parse(dml);

    let post_model = schema.assert_has_model("Post");
    post_model
        .assert_has_relation_field("user")
        .assert_relation_to("User")
        .assert_relation_referenced_fields(&["firstName"]);
}

#[test]
fn resolve_related_fields() {
    let dml = indoc! {r#"
        model User {
          id Int @id
          firstName String
          lastName String
          posts Post[]

          @@unique([firstName, lastName])
        }

        model Post {
          id Int @id
          text String
          authorFirstName String
          authorLastName  String
          user            User @relation(fields: [authorFirstName, authorLastName], references: [firstName, lastName])
        }
    "#};

    let schema = parse(dml);

    let post_model = schema.assert_has_model("Post");
    post_model
        .assert_has_relation_field("user")
        .assert_relation_to("User")
        .assert_relation_base_fields(&["authorFirstName", "authorLastName"])
        .assert_relation_referenced_fields(&["firstName", "lastName"]);
}

#[test]
fn allow_multiple_relations() {
    let dml = indoc! {r#"
        model User {
          id         Int    @id
          more_posts Post[] @relation(name: "more_posts")
          posts      Post[]
        }

        model Post {
          id            Int    @id
          text          String
          userId        Int
          postingUserId Int

          user         User   @relation(fields: userId, references: id)
          posting_user User   @relation(name: "more_posts", fields: postingUserId, references: id)
        }
    "#};

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model
        .assert_field_count(3)
        .assert_has_relation_field("posts")
        .assert_relation_to("Post")
        .assert_arity(&dml::FieldArity::List)
        .assert_relation_name("PostToUser");
    user_model
        .assert_has_relation_field("more_posts")
        .assert_relation_to("Post")
        .assert_arity(&dml::FieldArity::List)
        .assert_relation_name("more_posts");

    let post_model = schema.assert_has_model("Post");
    post_model
        .assert_field_count(6)
        .assert_has_scalar_field("text")
        .assert_base_type(&ScalarType::String);
    post_model
        .assert_has_relation_field("user")
        .assert_relation_to("User")
        .assert_arity(&dml::FieldArity::Required)
        .assert_relation_name("PostToUser");
    post_model
        .assert_has_relation_field("posting_user")
        .assert_relation_to("User")
        .assert_arity(&dml::FieldArity::Required)
        .assert_relation_name("more_posts");
}

#[test]
fn allow_complicated_self_relations() {
    let dml = indoc! {r#"
        model User {
          id     Int  @id
          sonId  Int?
          wifeId Int?

          son     User? @relation(name: "offspring", fields: sonId, references: id)
          father  User? @relation(name: "offspring")

          husband User? @relation(name: "spouse")
          wife    User? @relation(name: "spouse", fields: wifeId, references: id)
        }
    "#};

    let schema = parse(dml);
    let user_model = schema.assert_has_model("User");
    user_model.assert_has_relation_field("son").assert_relation_to("User");
    user_model
        .assert_has_relation_field("father")
        .assert_relation_to("User");
    user_model
        .assert_has_relation_field("husband")
        .assert_relation_to("User");
    user_model.assert_has_relation_field("wife").assert_relation_to("User");
}

#[test]
fn allow_explicit_fk_name_definition() {
    let dml = indoc! {r#"
        model User {
          user_id Int    @id
          posts   Post[]
        }

        model Post {
          post_id Int    @id
          user_id Int
          user    User    @relation(fields: user_id, references: user_id, map: "CustomFKName")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Postgres, &[]));

    schema
        .assert_has_model("User")
        .assert_has_relation_field("posts")
        .assert_relation_fk_name(None);

    schema
        .assert_has_model("Post")
        .assert_has_relation_field("user")
        .assert_relation_referenced_fields(&["user_id"])
        .assert_relation_fk_name(Some("CustomFKName".to_string()));
}

#[test]
fn allow_implicit_fk_name_definition() {
    let dml = indoc! {r#"
        model User {
          user_id Int    @id
          posts   Post[]
        }

        model Post {
          post_id Int    @id
          user_id Int
          user    User    @relation(fields: user_id, references: user_id)
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Postgres, &[]));

    schema
        .assert_has_model("User")
        .assert_has_relation_field("posts")
        .assert_relation_fk_name(None);
    schema
        .assert_has_model("Post")
        .assert_has_relation_field("user")
        .assert_relation_referenced_fields(&["user_id"])
        .assert_relation_fk_name(Some("Post_user_id_fkey".to_string()));
}

#[test]
fn implicit_fk_name_definition_with_mapped_models_and_fields() {
    let dml = indoc! {r#"
        model User {
          user_id Int    @id  @map("user_id_map")
          posts   Post[]

          @@map("UserMap")
        }

        model Post {
          user    User    @relation(fields: user_id, references: user_id)
          post_id Int    @id @map("post_id_map")
          user_id Int        @map("user_id_map_on_post")

          @@map("PostMap")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Postgres, &[]));

    schema
        .assert_has_model("User")
        .assert_has_relation_field("posts")
        .assert_relation_fk_name(None);

    schema
        .assert_has_model("Post")
        .assert_has_relation_field("user")
        .assert_relation_referenced_fields(&["user_id"])
        .assert_relation_fk_name(Some("PostMap_user_id_map_on_post_fkey".to_string()));
}

#[test]
fn implicit_fk_name_definition_with_mapped_models_and_fields_other_order() {
    let dml = indoc! {r#"
        model User {
          user_id Int    @id  @map("user_id_map")
          posts   Post[]

          @@map("UserMap")
        }

        model Post {
          post_id Int  @id @map("post_id_map")
          user_id Int  @map("user_id_map_on_post")
          user    User @relation(fields: user_id, references: user_id)

          @@map("PostMap")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Postgres, &[]));

    schema
        .assert_has_model("User")
        .assert_has_relation_field("posts")
        .assert_relation_fk_name(None);

    schema
        .assert_has_model("Post")
        .assert_has_relation_field("user")
        .assert_relation_referenced_fields(&["user_id"])
        .assert_relation_fk_name(Some("PostMap_user_id_map_on_post_fkey".to_string()));
}

#[test]
fn implicit_unique_constraint_on_one_to_one() {
    let dml = indoc! {r#"
        model User {
          user_id Int    @id  @map("user_id_map")
          post    Post?

          @@map("UserMap")
        }

        model Post {
          post_id Int    @id @map("post_id_map")
          user_id Int    @map("user_id_map_on_post")
          user    User   @relation(fields: user_id, references: user_id)

          @@map("PostMap")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Postgres, &[]));

    schema
        .assert_has_model("User")
        .assert_has_relation_field("post")
        .assert_relation_fk_name(None);

    schema
        .assert_has_model("Post")
        .assert_has_relation_field("user")
        .assert_relation_referenced_fields(&["user_id"])
        .assert_relation_fk_name(Some("PostMap_user_id_map_on_post_fkey".to_string()));

    schema.assert_has_model("Post").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("PostMap_user_id_map_on_post_key".to_string()),
        fields: vec![IndexField::new_in_model("user_id")],
        tpe: IndexType::Unique,
        defined_on_field: true,
        algorithm: None,
        clustered: None,
    });
}

#[test]
fn implicit_unique_constraint_on_compound_one_to_one() {
    let dml = indoc! {r#"
        model User {
          user_id_1  Int
          user_id_2  Int
          post       Post?

          @@id([user_id_1, user_id_2])
        }

        model Post {
            post_id    Int  @id
            user_id_1  Int
            user_id_2  Int
            user       User @relation(fields: [user_id_1, user_id_2], references: [user_id_1, user_id_2])
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Postgres, &[]));

    schema
        .assert_has_model("User")
        .assert_has_relation_field("post")
        .assert_relation_fk_name(None);

    schema
        .assert_has_model("Post")
        .assert_has_relation_field("user")
        .assert_relation_referenced_fields(&["user_id_1", "user_id_2"])
        .assert_relation_fk_name(Some("Post_user_id_1_user_id_2_fkey".to_string()));

    schema.assert_has_model("Post").assert_has_index(IndexDefinition {
        name: None,
        db_name: Some("Post_user_id_1_user_id_2_key".to_string()),
        fields: vec![
            IndexField::new_in_model("user_id_1"),
            IndexField::new_in_model("user_id_2"),
        ],
        tpe: IndexType::Unique,
        defined_on_field: false,
        algorithm: None,
        clustered: None,
    });
}

#[test]
fn no_unique_constraint_if_referring_the_pk() {
    let dml = indoc! {r#"
        model Cat {
          id      Int @id
          collar  Collar?
        }

        model Collar {
          id      Int @id
          cat     Cat @relation(fields:[id], references: [id])
        }
    "#};

    let dml = render_datamodel_to_string(&parse(&with_header(dml, Provider::Postgres, &[])), None);

    let expected = expect![[r#"
        model Cat {
          id     Int     @id
          collar Collar?
        }

        model Collar {
          id  Int @id
          cat Cat @relation(fields: [id], references: [id])
        }
    "#]];

    expected.assert_eq(&dml);
}

#[test]
fn one_to_one_optional() {
    let dml = indoc! {r#"
        model A {
          id Int @id
          b  B?
        }

        model B {
          id   Int @id
          a_id Int?
          a    A? @relation(fields: [a_id], references: [id])
        }
    "#};

    let schema = parse(dml);
    schema.assert_has_model("A").assert_has_relation_field("b");
    schema.assert_has_model("B").assert_has_relation_field("a");
}

#[test]
fn embedded_many_to_many_relations_work_on_mongodb() {
    let dml = indoc! {r#"
        model A {
          id    String   @id @map("_id") @default(auto()) @test.ObjectId
          b_ids String[] @test.ObjectId
          bs    B[]      @relation(fields: [b_ids], references: [id])
        }

        model B {
          id    String   @id @map("_id") @default(auto()) @test.ObjectId
          a_ids String[] @test.ObjectId
          as    A[]      @relation(fields: [a_ids], references: [id])
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Mongo, &[]));

    schema
        .assert_has_model("A")
        .assert_has_relation_field("bs")
        .assert_relation_base_fields(&["b_ids"])
        .assert_relation_referenced_fields(&["id"]);

    schema
        .assert_has_model("B")
        .assert_has_relation_field("as")
        .assert_relation_base_fields(&["a_ids"])
        .assert_relation_referenced_fields(&["id"]);
}

#[test]
fn implicit_many_to_many_relations_work_on_postgresql() {
    let dml = indoc! {r#"
        model A {
          id Int @id
          bs B[] @relation("foo")
        }

        model B {
          id Int @id
          as A[] @relation("foo")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Postgres, &[]));
    schema.assert_has_model("A").assert_has_relation_field("bs");
    schema.assert_has_model("B").assert_has_relation_field("as");
}

#[test]
fn implicit_many_to_many_relations_work_on_mysql() {
    let dml = indoc! {r#"
        model A {
          id Int @id
          bs B[] @relation("foo")
        }

        model B {
          id Int @id
          as A[] @relation("foo")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Mysql, &[]));
    schema.assert_has_model("A").assert_has_relation_field("bs");
    schema.assert_has_model("B").assert_has_relation_field("as");
}

#[test]
fn implicit_many_to_many_relations_work_on_sql_server() {
    let dml = indoc! {r#"
        model A {
          id Int @id
          bs B[] @relation("foo")
        }

        model B {
          id Int @id
          as A[] @relation("foo")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::SqlServer, &[]));
    schema.assert_has_model("A").assert_has_relation_field("bs");
    schema.assert_has_model("B").assert_has_relation_field("as");
}

#[test]
fn implicit_many_to_many_relations_work_on_sqlite() {
    let dml = indoc! {r#"
        model A {
          id Int @id
          bs B[] @relation("foo")
        }

        model B {
          id Int @id
          as A[] @relation("foo")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Sqlite, &[]));
    schema.assert_has_model("A").assert_has_relation_field("bs");
    schema.assert_has_model("B").assert_has_relation_field("as");
}

#[test]
fn implicit_many_to_many_relations_work_on_cockroach() {
    let dml = indoc! {r#"
        model A {
          id Int @id
          bs B[] @relation("foo")
        }

        model B {
          id Int @id
          as A[] @relation("foo")
        }
    "#};

    let schema = parse(&with_header(dml, Provider::Cockroach, &["cockroachDb"]));
    schema.assert_has_model("A").assert_has_relation_field("bs");
    schema.assert_has_model("B").assert_has_relation_field("as");
}
