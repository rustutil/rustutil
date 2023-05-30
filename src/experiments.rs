use crate::Experiment;

pub fn has_experiments(experiments: &Option<Vec<Experiment>>) -> bool {
    experiments.is_some()
}

pub fn has_experiment(experiments: &Option<Vec<Experiment>>, experiment: &Experiment) -> bool {
    has_experiments(experiments) && experiments.as_ref().unwrap().contains(experiment)
}
