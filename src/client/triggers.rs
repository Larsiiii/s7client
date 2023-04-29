use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use crate::errors::Error;
use crate::{S7Pool, S7ReadAccess};

#[derive(Debug)]
struct PLCBool {
    value: bool,
    last_known_value: bool,
}

impl PLCBool {
    fn new(start: bool) -> Self {
        Self {
            value: start,
            last_known_value: start,
        }
    }

    fn update(&mut self, new_value: bool) {
        self.last_known_value = self.value;
        self.value = new_value;
    }

    fn positive_flank(&self) -> bool {
        self.value && !self.last_known_value
    }

    fn negative_flank(&self) -> bool {
        !self.value && self.last_known_value
    }
}

/// Collection of observed `Bool` variables of the PLC
pub struct TriggerCollection<T>
where
    T: Hash + Eq,
{
    stored_values: HashMap<T, PLCBool>,
    plc_values: Vec<S7ReadAccess>,
    value_ids: Vec<T>,
    pool: S7Pool,
}

impl<T> Debug for TriggerCollection<T>
where
    T: Hash + Eq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TriggerCollection")
            .field("observed plc values", &self.plc_values)
            .finish()
    }
}

impl<T> TriggerCollection<T>
where
    T: Hash + Eq + Clone,
{
    pub(crate) fn new(pool: &S7Pool, triggers: &[(T, S7ReadAccess)]) -> Result<Self, Error> {
        let value_ids: Vec<T> = triggers
            .iter()
            .map(|trigger| trigger.0.to_owned())
            .collect();

        let plc_values: Vec<S7ReadAccess> = triggers.iter().map(|trigger| trigger.1).collect();

        // ensure that only bits are in ReadAccess vec
        if plc_values.iter().any(|read_access| match read_access {
            S7ReadAccess::Bytes { .. } => true,
            S7ReadAccess::Bit { .. } => false,
        }) {
            // throw error because Bytes are tried to be read
            return Err(Error::InvalidTriggerCollection);
        };

        let mut stored_values = HashMap::new();

        for id in &value_ids {
            stored_values.insert(id.to_owned(), PLCBool::new(false));
        }

        Ok(Self {
            stored_values,
            plc_values,
            value_ids,
            pool: pool.clone(),
        })
    }

    /// Read current values from PLC and update collection of observed `Bool` variables
    /// # Errors
    ///
    /// Will return `Error` if the `TriggerCollection` could not be updated.
    pub async fn update(&mut self) -> Result<(), Error> {
        let values = self.pool.db_read_multi(&self.plc_values).await?;

        for (index, value) in values.into_iter().enumerate() {
            let bool = value?[0] > 0;
            let trigger_id = &self.value_ids[index];

            // Should always be true!
            if let Some(trigger) = self.stored_values.get_mut(trigger_id) {
                trigger.update(bool);
            }
        }

        Ok(())
    }

    /// Check one of the observed triggers for a positive flank compared to before the last update of the collection.
    ///
    /// Returns `Some(true)` if positive flank is detected.
    ///
    /// Returns `Some(false)` if no change is detected.
    ///
    /// Returns `None` if given trigger is not part of the collection.
    pub fn positive_flank<Q>(&self, trigger: &Q) -> Option<bool>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.stored_values.get(trigger).map(PLCBool::positive_flank)
    }

    /// Check one of the observed triggers for a negative flank compared to before the last update of the collection.
    ///
    /// Returns `Some(true)` if negative flank is detected.
    ///
    /// Returns `Some(false)` if no change is detected.
    ///
    /// Returns `None` if given trigger is not part of the collection.
    pub fn negative_flank<Q>(&self, trigger: &Q) -> Option<bool>
    where
        T: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.stored_values.get(trigger).map(PLCBool::negative_flank)
    }
}
