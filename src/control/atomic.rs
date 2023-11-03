//! Helpers for atomic modesetting.

use crate::control;

/// Helper struct to construct atomic commit requests
#[derive(Debug, Clone, Default)]
pub struct AtomicModeReq {
    pub(super) objects: Vec<control::RawResourceHandle>,
    pub(super) count_props_per_object: Vec<u32>,
    pub(super) props: Vec<control::property::Handle>,
    pub(super) values: Vec<control::property::RawValue>,
}

impl AtomicModeReq {
    /// Create a new and empty atomic commit request
    pub fn new() -> AtomicModeReq {
        Self::default()
    }

    /// Add a property and value pair for a given raw resource to the request
    pub fn add_raw_property(
        &mut self,
        obj_id: control::RawResourceHandle,
        prop_id: control::property::Handle,
        value: control::property::RawValue,
    ) {
        // add object if missing (also to count_props_per_object)
        let (idx, prop_count) = match self.objects.binary_search(&obj_id) {
            Ok(idx) => (idx, self.count_props_per_object[idx]),
            Err(new_idx) => {
                self.objects.insert(new_idx, obj_id);
                self.count_props_per_object.insert(new_idx, 0);
                (new_idx, 0)
            }
        };

        // get start of our objects props
        let prop_slice_start = self.count_props_per_object.iter().take(idx).sum::<u32>() as usize;
        // get end
        let prop_slice_end = prop_slice_start + prop_count as usize;

        // search for existing prop entry
        match self.props[prop_slice_start..prop_slice_end]
            .binary_search_by_key(&Into::<u32>::into(prop_id), |x| (*x).into())
        {
            // prop exists, override
            Ok(prop_idx) => {
                self.values[prop_slice_start + prop_idx] = value;
            }
            Err(prop_idx) => {
                // increase prop count
                self.count_props_per_object[idx] += 1;
                // insert prop, insert value
                self.props.insert(prop_slice_start + prop_idx, prop_id);
                self.values.insert(prop_slice_start + prop_idx, value);
            }
        }
    }

    /// Add a property and value pair for a given handle to the request
    pub fn add_property<H>(
        &mut self,
        handle: H,
        property: control::property::Handle,
        value: control::property::Value,
    ) where
        H: control::ResourceHandle,
    {
        self.add_raw_property(handle.into(), property, value.into())
    }
}
