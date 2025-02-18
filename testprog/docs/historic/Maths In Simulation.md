# Maths In Simulation.md

## Introduction

This document outlines the requirements for a comprehensive mathematics library designed for the `testprog` engine. The library will provide a foundation for various mathematical operations, with a focus on applications in graphics, mechanical engineering, physics simulations, and modeling.

## General Purpose Mathematics

This section covers fundamental mathematical functions and structures.

- **Basic Arithmetic:**
  - Support for integers, floating-point numbers (single and double precision).
  - Addition, subtraction, multiplication, division.
  - Modulo operation, power, square root, absolute value.
- **Linear Algebra:**
  - Vectors (2D, 3D, 4D):
    - Addition, subtraction, scalar multiplication, dot product, cross product, normalization, magnitude.
  - Matrices (2x2, 3x3, 4x4):
    - Addition, subtraction, scalar multiplication, matrix multiplication, transpose, inverse, determinant.
  - Quaternions (for representing rotations):
    - Creation, normalization, conjugate, inverse, multiplication, slerp (spherical linear interpolation).
- **Trigonometry:**
  - Sine, cosine, tangent, and their inverses (arcsin, arccos, arctan).
  - Hyperbolic functions (sinh, cosh, tanh).
  - Angle conversions (degrees to radians, radians to degrees).
- **Calculus:**
  - Numerical differentiation and integration.
  - Support for common functions (polynomials, exponentials, logarithms).
- **Statistics:**
  - Mean, median, mode, standard deviation, variance.
  - Random number generation (uniform, normal distributions).
- **Geometry:**
  - Point, Line, Plane, Sphere, Triangle, Rectangle, AABB (Axis-Aligned Bounding Box), OBB (Oriented Bounding Box) representations.
  - Intersection tests (e.g., line-plane, sphere-AABB).
  - Distance calculations.

## Graphics Computing

This section focuses on mathematical operations specific to computer graphics.

- **Transformations:**
  - Translation, rotation, scaling using matrices and quaternions.
  - View and projection matrices (perspective and orthographic).
  - Model-View-Projection (MVP) matrix calculations.
  - Normal matrix calculation.
- **Color Operations:**
  - Representation of colors (RGBA, HSV).
  - Color blending and manipulation.
- **Interpolation:**
  - Linear interpolation (lerp).
  - Bilinear and trilinear interpolation.
  - Bezier curves and surfaces.
  - Splines (e.g., Catmull-Rom, B-splines).
- **Ray Tracing:**
  - Ray-primitive intersection tests (ray-sphere, ray-triangle, ray-AABB).

## Mechanical Mathematics

This section covers mathematical concepts relevant to mechanical engineering.

- **Thermodynamics:**
  - Rankine cycle analysis:
    - Ideal and non-ideal cycles.
    - Calculations for enthalpy, entropy, specific heat, work, and efficiency.
    - Functions for steam properties (using steam tables or appropriate approximations).
- **Turbine Analysis:**
  - Velocity triangles for turbine blades.
  - Calculations for blade angles, flow velocities, work done, and efficiency.
  - Analysis of different turbine types (impulse, reaction).
- **Fluid Dynamics:**
  - Basic calculations for pressure, flow rate, and velocity.
  - Bernoulli's principle.
- **Stress and Strain:**
  - Calculations for stress, strain, and deformation in materials.
  - Young's modulus, Poisson's ratio.

## Physics and Simulation

This section details the mathematical requirements for physics simulations.

- **Kinematics:**
  - Calculations for position, velocity, and acceleration.
  - Projectile motion.
  - Rotational motion (angular velocity, angular acceleration, torque).
- **Dynamics:**
  - Newton's laws of motion.
  - Force calculations (gravity, friction, spring forces).
  - Collision detection and response:
    - Impulse calculations.
    - Coefficient of restitution.
- **Numerical Integration:**
  - Euler method.
  - Verlet integration.
  - Runge-Kutta methods (RK4).
- **Particle Systems:**
  - Emitters, forces, and particle behavior.

## Implementation Details

- **Language:** Rust.
- **Performance:** The library should be highly optimized for performance. Consider using SIMD (Single Instruction, Multiple Data) instructions where appropriate.
- **Testing:** Comprehensive unit tests should be included to ensure correctness and stability.
- **Documentation:** Clear and concise documentation with examples.
- **Modularity:** The library should be modular, allowing users to include only the parts they need.
- **Dependencies:** Minimize external dependencies.

## Future Considerations

- **Advanced Numerical Methods:** Finite element analysis (FEA), computational fluid dynamics (CFD).
- **GPU Integration:** Leverage GPU acceleration for computationally intensive operations.
- **Scripting Integration:** Expose the library to a scripting language for easier prototyping and experimentation.
