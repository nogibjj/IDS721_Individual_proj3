use lambda_runtime::{run, service_fn, Error, LambdaEvent};

use serde::{Deserialize, Serialize};

use smartcore::dataset::breast_cancer;
use smartcore::linalg::naive::dense_matrix::DenseMatrix;
// use smartcore::math::distance::euclidian::Euclidian;
// use smartcore::metrics::accuracy;
use smartcore::model_selection::train_test_split;
use smartcore::neighbors::knn_classifier::KNNClassifier;

#[derive(Deserialize)]
struct Request {}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Load the Wine dataset
    let breast_cancer_data = breast_cancer::load_dataset();
    // Split the dataset into the input matrix (X) and output/target (y)
    let x = DenseMatrix::from_array(breast_cancer_data.num_samples, breast_cancer_data.num_features, &breast_cancer_data.data);
    let y = breast_cancer_data.target;

    // Split the dataset into a training and testing set (80% train, 20% test)
    let (x_train, x_test, y_train, y_test) = train_test_split(&x, &y, 0.2, true);

    // Create a k-NN classifier with k = 5 and Euclidean distance
    let knn = KNNClassifier::fit(&x_train, &y_train,Default::default()).unwrap();

    // Predict the cancer class for the test dataset
    let y_pred = knn.predict(&x_test).unwrap();

    // Calculate and print the accuracy
    let accuracy = y_pred
        .iter()
        .zip(y_test.iter())
        .map(|(y_pred, y_true)| usize::from(y_pred == y_true))
        .sum::<usize>() as f32
        / y_test.len() as f32;

    let message = format!("accuracy: {}", accuracy);
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("{}.", message),
    };
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
