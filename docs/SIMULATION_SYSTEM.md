# Evolution Simulation System Documentation

## Overview

The Evolution Simulation is a complex ecosystem simulation built in Rust using an Entity-Component-System (ECS) architecture. It simulates the evolution of entities through natural selection, genetic inheritance, and environmental pressures.

## Core Architecture

### Technology Stack

- **Language**: Rust
- **ECS Framework**: Hecs
- **Parallelism**: Rayon
- **Rendering**: WebGPU (for web) / WGPU (for desktop)
- **Serialization**: Serde
- **Random Number Generation**: Rand

### Design Principles

- **Minimize special cases**: Systems are focused on specific component sets
- **Parallel processing**: CPU-intensive operations use Rayon for multi-core utilization
- **Scalability**: Designed for large populations (10,000+ entities)
- **Modularity**: Systems are independent and focused on specific behaviors
- **Configurability**: All simulation parameters are easily adjustable

## Current System Components

### 1. Entity Components

#### Core Components

- **Position**: 2D coordinates (x, y)
- **Velocity**: Movement vector (x, y)
- **Energy**: Current and maximum energy levels
- **Size**: Radius affecting interactions and energy costs
- **Color**: Visual representation based on genes
- **Genes**: Complete genetic blueprint

#### Movement Components

- **MovementStyle**: Configurable movement behavior
  - `style`: Movement type (Random, Flocking, Solitary, Predatory, Grazing)
  - `flocking_strength`: How strongly to flock (0.0-1.0)
  - `separation_distance`: Preferred distance from flock members
  - `alignment_strength`: How much to align with flock direction
  - `cohesion_strength`: How much to move toward flock center

### 2. Genetic System

#### Gene Categories

- **MovementGenes**

  - `speed`: Movement velocity (0.1-2.5)
  - `sense_radius`: Detection range (5.0-150.0)

- **EnergyGenes**

  - `efficiency`: Energy conversion efficiency (0.3-3.0)
  - `loss_rate`: Energy consumption rate (0.05-2.0)
  - `gain_rate`: Energy absorption rate (0.2-4.5)
  - `size_factor`: Size scaling factor (0.3-2.5)

- **ReproductionGenes**

  - `rate`: Reproduction probability (0.0005-0.15)
  - `mutation_rate`: Gene mutation probability (0.005-0.15)

- **AppearanceGenes**

  - `hue`: Color hue (0.0-1.0)
  - `saturation`: Color saturation (0.2-1.0)

- **BehaviorGenes**
  - `movement_style`: Movement behavior configuration
  - `gene_preference_strength`: Predation preference for different genes (0.0-1.0)
  - `social_tendency`: Social vs solitary behavior (0.0-1.0)

#### Genetic Operations

- **Mutation**: Random changes to gene values during reproduction
- **Inheritance**: Offspring inherit parent genes with mutations
- **Gene Similarity**: Calculated across all gene categories
- **Predation Preference**: Entities prefer eating genetically different prey

### 3. Movement System

#### Movement Types

1. **Random**: Standard random movement with speed variation
2. **Flocking**: Group behavior with cohesion, alignment, and separation
3. **Solitary**: Avoids other entities
4. **Predatory**: Hunts for preferred prey
5. **Grazing**: Slow, steady movement with gentle randomness

#### Flocking Algorithm

- **Cohesion**: Move toward flock center
- **Alignment**: Align direction with flock average
- **Separation**: Maintain distance from flock members
- **Gene-based flocking**: Only flock with genetically similar entities

### 4. Interaction System

#### Predation Mechanics

- **Size advantage**: Larger entities can eat smaller ones
- **Speed advantage**: Faster entities have hunting advantage
- **Gene preference**: Entities prefer eating genetically different prey
- **Energy gain**: More energy from preferred prey (up to 50% bonus)

#### Interaction Rules

- Entities must be within interaction radius
- Prey must have positive energy
- Only one interaction per frame per entity
- Eaten entities are removed from simulation

### 5. Energy System

#### Energy Dynamics

- **Movement cost**: Energy consumed based on distance moved
- **Size cost**: Larger entities cost more to maintain
- **Efficiency**: Gene-based energy conversion efficiency
- **Gain**: Energy obtained from eating other entities

#### Energy Constraints

- Entities die when energy reaches zero
- Maximum energy based on efficiency genes
- Energy affects entity size
- Reproduction requires high energy levels

### 6. Reproduction System

#### Reproduction Conditions

- Energy above threshold (80% of maximum)
- Population density below limit
- Random chance based on reproduction rate gene
- Parent energy cost for reproduction

#### Offspring Creation

- Inherit parent genes with mutations
- Spawn near parent location
- Reduced initial energy
- Inherit movement style and behavior genes

### 7. Spatial System

#### Spatial Grid

- Divides world into cells for efficient neighbor queries
- Configurable cell size (default: 25.0)
- Handles entity insertion, removal, and neighbor detection
- Rebuilt each frame to maintain accuracy

#### Boundary Handling

- Entities bounce off world boundaries
- Center pressure keeps entities away from edges
- Configurable boundary margins and bounce factors

### 8. Configuration System

#### Simulation Parameters

- **Population**: Entity counts, spawn radius, density factors
- **Physics**: Velocities, sizes, interaction distances, boundary settings
- **Energy**: Consumption rates, size factors, movement costs
- **Reproduction**: Thresholds, costs, mutation rates, density factors

#### Configuration Management

- JSON-based configuration files
- Default configurations
- Runtime parameter adjustment
- Save/load functionality

## Current Simulation Features

### 1. Natural Selection

- Entities with better genes survive longer
- Successful traits are passed to offspring
- Population adapts to environmental pressures
- Genetic diversity through mutation

### 2. Complex Behaviors

- Five distinct movement types
- Gene-based flocking behavior
- Intelligent predation preferences
- Social vs solitary tendencies

### 3. Ecosystem Dynamics

- Predator-prey relationships
- Population density effects
- Energy-based survival
- Genetic diversity maintenance

### 4. Visual Representation

- Color-coded entities based on genes
- Smooth movement interpolation
- Real-time statistics display
- Web and desktop rendering

### 5. Performance Optimization

- Parallel entity processing
- Spatial partitioning for neighbor queries
- Efficient ECS queries
- Scalable to large populations

## Statistics and Monitoring

### Entity Metrics

- Population counts by movement type
- Average gene values
- Energy distribution
- Size distribution

### System Metrics

- Frame rate and performance
- Memory usage
- Entity processing times
- Spatial grid efficiency

### Evolution Tracking

- Gene value trends over time
- Population diversity measures
- Survival rate statistics
- Reproduction success rates

## Current Limitations

### Technical Limitations

- 2D simulation only
- No environmental obstacles
- Limited sensory systems
- No memory or learning

### Biological Limitations

- No disease or parasites
- No aging or life stages
- No social hierarchies
- No territorial behavior

### Ecological Limitations

- Single species simulation
- No environmental variation
- No seasonal changes
- No resource competition

## Future Development Ideas

### 1. Environmental Factors

- **Terrain/obstacles**: Add impassable areas that entities must navigate around
- **Resource patches**: Areas with different energy regeneration rates
- **Temperature zones**: Affect energy consumption and reproduction rates
- **Day/night cycles**: Change visibility, movement speed, and behavior patterns

### 2. Advanced Behaviors

- **Mating rituals**: Entities need to perform specific behaviors to reproduce
- **Territorial behavior**: Entities defend areas and chase away intruders
- **Learning/adaptation**: Entities remember successful strategies and avoid failed ones
- **Communication**: Entities can signal to each other (warning calls, mating calls)
- **Tool use**: Some entities can use environmental objects as weapons or tools

### 3. Disease and Parasites

- **Infectious diseases**: Spread between entities in close proximity
- **Parasites**: Reduce energy efficiency and can be passed to offspring
- **Immunity genes**: Some entities are naturally resistant to certain diseases
- **Quarantine behavior**: Sick entities might isolate themselves

### 4. Social Structures

- **Hierarchies**: Alpha entities that others follow or avoid
- **Families**: Parent-child bonds that affect behavior
- **Tribes/clans**: Groups that cooperate and share resources
- **Alliances**: Temporary partnerships between entities

### 5. Advanced Genetics

- **Epigenetics**: Environmental factors affecting gene expression
- **Genetic disorders**: Harmful mutations that reduce fitness
- **Hybridization**: Different species can sometimes interbreed
- **Gene transfer**: Horizontal gene transfer between living entities

### 6. Ecosystem Dynamics

- **Multiple species**: Different types of entities with distinct behaviors
- **Food webs**: Complex predator-prey relationships
- **Symbiosis**: Mutualistic relationships between entities
- **Invasive species**: New entity types that disrupt existing ecosystems

### 7. Environmental Challenges

- **Natural disasters**: Periodic events that affect large areas
- **Climate change**: Gradual environmental shifts
- **Migration**: Seasonal movement patterns
- **Hibernation**: Periods of reduced activity to conserve energy

### 8. Advanced Movement

- **Swimming/flying**: Different movement modes for different environments
- **Climbing**: Ability to traverse vertical surfaces
- **Burrowing**: Underground movement and hiding
- **Gliding**: Efficient long-distance movement

### 9. Sensory Systems

- **Vision**: Line of sight affects what entities can detect
- **Hearing**: Sound-based detection and communication
- **Smell**: Chemical trails and pheromones
- **Touch**: Physical contact detection

### 10. Life Cycle Stages

- **Egg/larval stages**: Vulnerable early life phases
- **Metamorphosis**: Dramatic physical changes
- **Aging**: Gradual decline in capabilities over time
- **Menopause**: End of reproductive capability

### 11. Advanced Reproduction

- **Sexual selection**: Mates chosen based on specific traits
- **Courtship displays**: Complex mating behaviors
- **Parental care**: Investment in offspring survival
- **Brood parasitism**: Laying eggs in others' nests

### 12. Memory and Intelligence

- **Spatial memory**: Remember locations of resources and dangers
- **Social memory**: Remember interactions with specific entities
- **Problem solving**: Ability to overcome obstacles
- **Innovation**: Discovering new behaviors through experimentation

### 13. Emotional States

- **Fear**: Affects movement and decision making
- **Aggression**: Influences combat and territorial behavior
- **Curiosity**: Drives exploration and learning
- **Stress**: Affects health and reproduction

### 14. Advanced Combat

- **Weapon evolution**: Natural weapons like claws, teeth, armor
- **Combat strategies**: Different fighting styles
- **Injury system**: Wounds that take time to heal
- **Retreat behavior**: Knowing when to flee

### 15. Resource Management

- **Fat storage**: Energy reserves for lean times
- **Water needs**: Additional resource to manage
- **Nutrient requirements**: Different types of "food"
- **Waste production**: Environmental pollution effects

## Implementation Priority

### High Priority (Core Gameplay)

1. **Territorial behavior** - Entities defend areas and have home ranges
2. **Environmental obstacles** - Makes movement and survival more challenging
3. **Disease system** - Adds another layer of natural selection
4. **Multiple species** - Creates more complex ecosystem dynamics

### Medium Priority (Enhanced Realism)

1. **Aging system** - Life stages and gradual decline
2. **Sensory systems** - Line of sight and hearing
3. **Social hierarchies** - Alpha entities and group dynamics
4. **Resource competition** - Limited food sources

### Low Priority (Advanced Features)

1. **Learning and memory** - Complex cognitive behaviors
2. **Tool use** - Environmental interaction
3. **Communication** - Signaling between entities
4. **Climate change** - Long-term environmental evolution

## Conclusion

The current simulation provides a solid foundation for evolutionary biology simulation with realistic genetic inheritance, natural selection, and complex behaviors. The modular architecture makes it easy to add new features while maintaining performance and scalability.

The future development ideas range from simple environmental additions to complex cognitive behaviors, allowing for gradual enhancement of the simulation's realism and complexity. Each addition should be carefully considered for its impact on performance and the overall simulation balance.
