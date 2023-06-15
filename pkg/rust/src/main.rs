#[cfg(feature = "sled-storage")]
mod sled_multi_threaded {
    use gluesql::{prelude::*, sled_storage::SledStorage};
    use gluesql_core::ast_builder::{generate_uuid, table, text, timestamp, Execute};

    pub async fn run() {
        let storage = SledStorage::new("./tmp/gluesql/user").expect("Something went wrong!");
        let mut glue = Glue::new(storage.clone());

        // create table
        let result = table("user")
            .create_table_if_not_exists()
            .add_column("id UUID PRIMARY KEY")
            .add_column("email TEXT UNIQUE")
            .add_column("password TEXT")
            .add_column("created_at TIMESTAMP")
            .execute(&mut glue)
            .await;
        assert_eq!(result, Ok(Payload::Create));

        // insert
        let result = table("user")
            .insert()
            .columns("id, email, password, created_at")
            .values(vec![vec![
                generate_uuid(),
                text("glue@example.com"),
                text("password"),
                timestamp("2020-01-01 00:00:00"),
            ]])
            .execute(&mut glue)
            .await;
        assert_eq!(result, Ok(Payload::Insert(1)));

        // select
        let query = "SELECT * FROM user";
        let results = glue.execute(query).await.unwrap();
        let rows = match &results[0] {
            Payload::Select { rows, .. } => rows,
            _ => panic!("Unexpected result: {:?}", results),
        };
        let first_row = &rows[0];
        assert_eq!(first_row.len(), 4);
        assert_eq!(first_row[1], Value::Str("glue@example.com".to_string()));

        // update
        let result = table("user")
            .update()
            .set("email", text("glue2@example.com"))
            .execute(&mut glue)
            .await;
        assert_eq!(result, Ok(Payload::Update(1)));

        // check exists
        let actual = table("user")
            .select()
            .filter("email = 'glue2@example.com'")
            .project("email")
            .execute(&mut glue)
            .await;
        assert_eq!(
            actual,
            Ok(Payload::Select {
                labels: vec!["email".to_string()],
                rows: vec![vec![Value::Str("glue2@example.com".to_string())]],
            })
        );

        // delete
        let result = table("user")
            .delete()
            .filter("email = 'glue2@example.com'")
            .execute(&mut glue)
            .await;
        assert_eq!(result, Ok(Payload::Delete(1)));

        // check not exists
        let actual = table("user")
            .select()
            .filter("email = 'glue2@example.com'")
            .execute(&mut glue)
            .await;
        assert_eq!(
            actual,
            Ok(Payload::Select {
                labels: vec![
                    "id".to_string(),
                    "email".to_string(),
                    "password".to_string(),
                    "created_at".to_string()
                ],
                rows: vec![]
            })
        );
    }
}

fn main() {
    #[cfg(feature = "sled-storage")]
    futures::executor::block_on(sled_multi_threaded::run());
}
