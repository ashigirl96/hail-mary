use crate::application::errors::ApplicationError;
use crate::application::repositories::SpecRepositoryInterface;

pub fn complete_specs(
    spec_repo: &dyn SpecRepositoryInterface,
    spec_names: &[String],
) -> Result<(), ApplicationError> {
    for name in spec_names {
        spec_repo.mark_spec_complete(name)?;
    }
    Ok(())
}
