



/**
 * A type for storing and manipulating parameters needed to build a container.
 */
pub struct ContainerParams {
    image: String,
}


impl ContainerParams {
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
        }
    }

    pub(crate) fn start_args(&self) -> Vec<String> {
        //TODO: The entire disposables setup should be here
        ["run", "-d", &self.image].map(|x| x.into()).into()
    }
}

