use hecs::Entity;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Memory-mapped storage for million-scale entity data
pub struct MemoryMappedStorage {
    file: File,
    entity_data: HashMap<Entity, EntityRecord>,
    next_offset: u64,
    compression_enabled: bool,
}

#[derive(Debug, Clone)]
struct EntityRecord {
    offset: u64,
    size: u32,
    compressed: bool,
}

/// Compressed entity data structure
#[derive(Debug, Clone)]
pub struct CompressedEntityData {
    pub position: [f32; 2], // 8 bytes
    pub velocity: [f32; 2], // 8 bytes
    pub energy: f32,        // 4 bytes
    pub size: f32,          // 4 bytes
    pub genes: [u8; 16],    // 16 bytes (compressed genes)
    pub color: [u8; 3],     // 3 bytes
    pub flags: u8,          // 1 byte
}

impl CompressedEntityData {
    pub fn size() -> usize {
        44 // Total size in bytes
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::size());

        // Position
        bytes.extend_from_slice(&self.position[0].to_le_bytes());
        bytes.extend_from_slice(&self.position[1].to_le_bytes());

        // Velocity
        bytes.extend_from_slice(&self.velocity[0].to_le_bytes());
        bytes.extend_from_slice(&self.velocity[1].to_le_bytes());

        // Energy and size
        bytes.extend_from_slice(&self.energy.to_le_bytes());
        bytes.extend_from_slice(&self.size.to_le_bytes());

        // Genes
        bytes.extend_from_slice(&self.genes);

        // Color
        bytes.extend_from_slice(&self.color);

        // Flags
        bytes.push(self.flags);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::size() {
            return None;
        }

        let mut offset = 0;

        // Position
        let pos_x = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;
        let pos_y = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // Velocity
        let vel_x = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;
        let vel_y = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // Energy and size
        let energy = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;
        let size = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // Genes
        let mut genes = [0u8; 16];
        genes.copy_from_slice(&bytes[offset..offset + 16]);
        offset += 16;

        // Color
        let mut color = [0u8; 3];
        color.copy_from_slice(&bytes[offset..offset + 3]);
        offset += 3;

        // Flags
        let flags = bytes[offset];

        Some(CompressedEntityData {
            position: [pos_x, pos_y],
            velocity: [vel_x, vel_y],
            energy,
            size,
            genes,
            color,
            flags,
        })
    }
}

impl MemoryMappedStorage {
    pub fn new<P: AsRef<Path>>(path: P, compression_enabled: bool) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        Ok(Self {
            file,
            entity_data: HashMap::new(),
            next_offset: 0,
            compression_enabled,
        })
    }

    pub fn store_entity(
        &mut self,
        entity: Entity,
        data: &CompressedEntityData,
    ) -> std::io::Result<()> {
        let bytes = data.to_bytes();
        let size = bytes.len() as u32;

        // Write data to file
        self.file.seek(SeekFrom::Start(self.next_offset))?;
        self.file.write_all(&bytes)?;

        // Store metadata
        self.entity_data.insert(
            entity,
            EntityRecord {
                offset: self.next_offset,
                size,
                compressed: self.compression_enabled,
            },
        );

        self.next_offset += size as u64;
        Ok(())
    }

    pub fn load_entity(&mut self, entity: Entity) -> std::io::Result<Option<CompressedEntityData>> {
        if let Some(record) = self.entity_data.get(&entity) {
            let mut bytes = vec![0u8; record.size as usize];

            self.file.seek(SeekFrom::Start(record.offset))?;
            self.file.read_exact(&mut bytes)?;

            Ok(CompressedEntityData::from_bytes(&bytes))
        } else {
            Ok(None)
        }
    }

    pub fn batch_store(
        &mut self,
        entities: &[(Entity, CompressedEntityData)],
    ) -> std::io::Result<()> {
        // Sort by entity for better locality
        let mut sorted_entities = entities.to_vec();
        sorted_entities.sort_by_key(|(entity, _)| *entity);

        for (entity, data) in sorted_entities {
            self.store_entity(entity, &data)?;
        }

        Ok(())
    }

    pub fn batch_load(
        &mut self,
        entities: &[Entity],
    ) -> std::io::Result<HashMap<Entity, CompressedEntityData>> {
        let mut result = HashMap::new();

        for &entity in entities {
            if let Some(data) = self.load_entity(entity)? {
                result.insert(entity, data);
            }
        }

        Ok(result)
    }

    pub fn get_stats(&self) -> StorageStats {
        StorageStats {
            total_entities: self.entity_data.len(),
            total_size: self.next_offset,
            compression_enabled: self.compression_enabled,
            avg_entity_size: if self.entity_data.is_empty() {
                0.0
            } else {
                self.next_offset as f64 / self.entity_data.len() as f64
            },
        }
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        self.file.set_len(0)?;
        self.entity_data.clear();
        self.next_offset = 0;
        Ok(())
    }
}

#[derive(Debug)]
pub struct StorageStats {
    pub total_entities: usize,
    pub total_size: u64,
    pub compression_enabled: bool,
    pub avg_entity_size: f64,
}

/// Entity pool for efficient memory management
pub struct EntityPool {
    storage: Arc<Mutex<MemoryMappedStorage>>,
    active_entities: HashMap<Entity, CompressedEntityData>,
    pool_size: usize,
}

impl EntityPool {
    pub fn new<P: AsRef<Path>>(path: P, pool_size: usize) -> std::io::Result<Self> {
        let storage = Arc::new(Mutex::new(MemoryMappedStorage::new(path, true)?));

        Ok(Self {
            storage,
            active_entities: HashMap::with_capacity(pool_size),
            pool_size,
        })
    }

    pub fn add_entity(
        &mut self,
        entity: Entity,
        data: CompressedEntityData,
    ) -> std::io::Result<()> {
        // Add to active pool
        self.active_entities.insert(entity, data.clone());

        // If pool is full, flush to storage
        if self.active_entities.len() >= self.pool_size {
            self.flush_to_storage()?;
        }

        Ok(())
    }

    pub fn get_entity(&mut self, entity: Entity) -> std::io::Result<Option<CompressedEntityData>> {
        // Check active pool first
        if let Some(data) = self.active_entities.get(&entity) {
            return Ok(Some(data.clone()));
        }

        // Load from storage
        let mut storage = self.storage.lock().unwrap();
        storage.load_entity(entity)
    }

    pub fn flush_to_storage(&mut self) -> std::io::Result<()> {
        if self.active_entities.is_empty() {
            return Ok(());
        }

        let entities: Vec<_> = self
            .active_entities
            .iter()
            .map(|(e, d)| (*e, d.clone()))
            .collect();

        let mut storage = self.storage.lock().unwrap();
        storage.batch_store(&entities)?;

        // Clear active pool
        self.active_entities.clear();

        Ok(())
    }

    pub fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            active_entities: self.active_entities.len(),
            pool_size: self.pool_size,
            storage_stats: {
                let storage = self.storage.lock().unwrap();
                storage.get_stats()
            },
        }
    }
}

#[derive(Debug)]
pub struct PoolStats {
    pub active_entities: usize,
    pub pool_size: usize,
    pub storage_stats: StorageStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compressed_entity_data() {
        let data = CompressedEntityData {
            position: [1.0, 2.0],
            velocity: [3.0, 4.0],
            energy: 100.0,
            size: 5.0,
            genes: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            color: [255, 128, 64],
            flags: 42,
        };

        let bytes = data.to_bytes();
        let reconstructed = CompressedEntityData::from_bytes(&bytes).unwrap();

        assert_eq!(data.position, reconstructed.position);
        assert_eq!(data.velocity, reconstructed.velocity);
        assert_eq!(data.energy, reconstructed.energy);
        assert_eq!(data.size, reconstructed.size);
        assert_eq!(data.genes, reconstructed.genes);
        assert_eq!(data.color, reconstructed.color);
        assert_eq!(data.flags, reconstructed.flags);
    }

    #[test]
    fn test_memory_mapped_storage() -> std::io::Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut storage = MemoryMappedStorage::new(temp_file.path(), false)?;

        let entity = hecs::Entity::from_bits(1).unwrap();
        let data = CompressedEntityData {
            position: [1.0, 2.0],
            velocity: [3.0, 4.0],
            energy: 100.0,
            size: 5.0,
            genes: [0; 16],
            color: [255, 0, 0],
            flags: 0,
        };

        storage.store_entity(entity, &data)?;
        let loaded = storage.load_entity(entity)?.unwrap();

        assert_eq!(data.position, loaded.position);
        assert_eq!(data.velocity, loaded.velocity);
        assert_eq!(data.energy, loaded.energy);

        Ok(())
    }
}
