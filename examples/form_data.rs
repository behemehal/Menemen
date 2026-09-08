//! Shows multiple ways to construct `FormData` values.

use std::collections::HashMap;

use menemen::form_data::FormData;

fn main() {
    let mut form_data_builder = FormData::new();
    form_data_builder.add("key1", "value1");
    form_data_builder.add("key2", "value2");

    println!("FormData from builder: {:?}", form_data_builder);

    let mut vec_form: Vec<(String, String)> = Vec::new();
    vec_form.push(("key1".to_string(), "value1".to_string()));
    vec_form.push(("key2".to_string(), "value2".to_string()));

    let form_data_from_vec: FormData = vec_form.into();

    println!("FormData from Vec: {:?}", form_data_from_vec);

    let mut hashmap_form: HashMap<String, String> = HashMap::new();
    hashmap_form.insert("key1".to_string(), "value1".to_string());
    hashmap_form.insert("key2".to_string(), "value2".to_string());

    let form_data_from_hashmap: FormData = hashmap_form.into();

    println!("FormData from HashMap: {:?}", form_data_from_hashmap);
}
