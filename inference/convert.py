"""Script to convert weights to Rust-compatible format."""

import re

import torch
from torch import Tensor, nn
from torch.distributions import Normal

DOF_NAMES = [
    "L_hip_y",
    "L_hip_x",
    "L_hip_z",
    "L_knee",
    "L_ankle_y",
    "R_hip_y",
    "R_hip_x",
    "R_hip_z",
    "R_knee",
    "R_ankle_y",
]

BODY_NAMES = [
    "base",
    "trunk",
    "L_buttock",
    "L_leg",
    "L_thigh",
    "L_calf",
    "L_foot",
    "L_clav",
    "L_scapula",
    "L_uarm",
    "L_farm",
    "R_buttock",
    "R_leg",
    "R_thigh",
    "R_calf",
    "R_foot",
    "R_clav",
    "R_scapula",
    "R_uarm",
    "R_farm",
]

DEFAULT_JOINT_ANGLES = {
    "L_ankle_y": -0.258,
    "L_hip_y": -0.157,
    "L_hip_z": 0.0628,
    "L_hip_x": 0.0394,
    "L_knee": 0.441,
    "R_ankle_y": -0.223,
    "R_hip_y": -0.22,
    "R_hip_z": 0.0314,
    "R_hip_x": 0.026,
    "R_knee": 0.441,
}

STIFFNESS = {"hip_y": 120, "hip_x": 60, "hip_z": 60, "knee": 120, "ankle_y": 17}
DAMPING = {"hip_y": 10, "hip_x": 10, "hip_z": 10, "knee": 10, "ankle_y": 5}

NUM_ACTIONS = len(DOF_NAMES)


class ActorCritic(nn.Module):
    def __init__(
        self,
        num_actor_obs: int,
        num_critic_obs: int,
        num_actions: int,
        actor_hidden_dims: list[int] = [256, 256, 256],
        critic_hidden_dims: list[int] = [256, 256, 256],
        init_noise_std: float = 1.0,
        activation: nn.Module = nn.ELU(),
    ) -> None:
        super(ActorCritic, self).__init__()

        mlp_input_dim_a = num_actor_obs
        mlp_input_dim_c = num_critic_obs

        # Policy function.
        actor_layers = []
        actor_layers.append(nn.Linear(mlp_input_dim_a, actor_hidden_dims[0]))
        actor_layers.append(activation)
        for dim_i in range(len(actor_hidden_dims)):
            if dim_i == len(actor_hidden_dims) - 1:
                actor_layers.append(nn.Linear(actor_hidden_dims[dim_i], num_actions))
            else:
                actor_layers.append(nn.Linear(actor_hidden_dims[dim_i], actor_hidden_dims[dim_i + 1]))
                actor_layers.append(activation)
        self.actor = nn.Sequential(*actor_layers)

        # Value function.
        critic_layers = []
        critic_layers.append(nn.Linear(mlp_input_dim_c, critic_hidden_dims[0]))
        critic_layers.append(activation)
        for dim_i in range(len(critic_hidden_dims)):
            if dim_i == len(critic_hidden_dims) - 1:
                critic_layers.append(nn.Linear(critic_hidden_dims[dim_i], 1))
            else:
                critic_layers.append(nn.Linear(critic_hidden_dims[dim_i], critic_hidden_dims[dim_i + 1]))
                critic_layers.append(activation)
        self.critic = nn.Sequential(*critic_layers)

        # Action noise.
        self.std = nn.Parameter(init_noise_std * torch.ones(num_actions))
        self.distribution = None

        # Disable args validation for speedup.
        Normal.set_default_validate_args = False


class Actor(nn.Module):
    def __init__(self, policy: nn.Module) -> None:
        super().__init__()

        self.policy = policy

        self.p_gains = torch.zeros(NUM_ACTIONS, dtype=torch.float)
        self.d_gains = torch.zeros(NUM_ACTIONS, dtype=torch.float)
        self.default_dof_pos = torch.zeros(NUM_ACTIONS, dtype=torch.float)

        for i in range(len(DOF_NAMES)):
            name = DOF_NAMES[i]
            self.default_dof_pos[i] = DEFAULT_JOINT_ANGLES[name]
            found = False

            for dof_name in STIFFNESS.keys():
                if dof_name in name:
                    self.p_gains[i] = STIFFNESS[dof_name]
                    self.d_gains[i] = DAMPING[dof_name]
                    found = True
            if not found:
                self.p_gains[i] = 0.0
                self.d_gains[i] = 0.0

        self.action_scale = 0.25
        self.lin_vel_scale = 2.0
        self.ang_vel_scale = 1.0
        self.quat_scale = 1.0
        self.dof_pos_scale = 1.0
        self.dof_vel_scale = 0.05

    def forward(
        self,
        x_vel: Tensor,
        y_vel: Tensor,
        rot: Tensor,
        t: Tensor,
        dof_pos: Tensor,
        dof_vel: Tensor,
        prev_actions: Tensor,
        imu_ang_vel: Tensor,
        imu_euler_xyz: Tensor,
        buffer: Tensor,
    ) -> tuple[Tensor, Tensor, Tensor]:
        """Runs the actor model forward pass.

        Args:
            x_vel: The x-coordinate of the target velocity, with shape (1).
            y_vel: The y-coordinate of the target velocity, with shape (1).
            rot: The target angular velocity, with shape (1).
            t: The current time, with shape (1).
            dof_pos: The current angular position of the DoFs, with shape (10).
            dof_vel: The current angular velocity of the DoFs, with shape (10).
            prev_actions: The previous actions taken by the model, with shape (10).
            imu_ang_vel: The angular velocity of the IMU, with shape (3),
                in radians per second. If IMU is not used, can be all zeros.
            imu_euler_xyz: The euler angles of the IMU, with shape (3),
                in radians. "XYZ" means (roll, pitch, yaw). If IMU is not used,
                can be all zeros.
            buffer: The buffer of previous actions, with shape (574). This is
                the return value of the previous forward pass. On the first
                pass, it should be all zeros.

        Returns:
            The torques to apply to the DoFs, the actions taken, and the
            next buffer.
        """
        phase = t * 0.02 / 0.5  # 50 Hz policy, 0.5 sec cycle time
        sin_pos = torch.sin(2 * torch.pi * phase)
        cos_pos = torch.cos(2 * torch.pi * phase)

        command_input = torch.cat(
            (
                sin_pos,
                cos_pos,
                x_vel * self.lin_vel_scale,
                y_vel * self.lin_vel_scale,
                rot * self.ang_vel_scale,
            ),
            dim=0,
        )

        q = (dof_pos - self.default_dof_pos) * self.dof_pos_scale
        dq = dof_vel * self.dof_vel_scale

        new_x = torch.cat(
            (
                command_input,
                q,
                dq,
                prev_actions,
                imu_ang_vel * self.ang_vel_scale,
                imu_euler_xyz * self.quat_scale,
            ),
            dim=0,
        )

        x = torch.cat((buffer, new_x), dim=0)

        actions = self.policy(x.unsqueeze(0)).squeeze(0)
        actions_scaled = actions * self.action_scale
        # p_gains = self.p_gains
        # d_gains = self.d_gains
        # torques = p_gains * (actions_scaled + self.default_dof_pos - dof_pos) - d_gains * dof_vel
        # return torques, actions, x[41:]
        return actions_scaled, actions, x[41:]


def convert() -> None:
    all_weights = torch.load("position_control.pt", map_location="cpu", weights_only=True)
    weights = all_weights["model_state_dict"]
    num_actor_obs = weights["actor.0.weight"].shape[1]
    num_critic_obs = weights["critic.0.weight"].shape[1]
    num_actions = weights["std"].shape[0]

    actor_hidden_dims = [v.shape[0] for k, v in weights.items() if re.match(r"actor\.\d+\.weight", k)]
    critic_hidden_dims = [v.shape[0] for k, v in weights.items() if re.match(r"critic\.\d+\.weight", k)]
    actor_hidden_dims = actor_hidden_dims[:-1]
    critic_hidden_dims = critic_hidden_dims[:-1]

    ac_model = ActorCritic(num_actor_obs, num_critic_obs, num_actions, actor_hidden_dims, critic_hidden_dims)
    ac_model.load_state_dict(weights)

    a_model = Actor(ac_model.actor)

    # Gets the model input tensors.
    x_vel = torch.randn(1)
    y_vel = torch.randn(1)
    rot = torch.randn(1)
    t = torch.randn(1)
    dof_pos = torch.randn(NUM_ACTIONS)
    dof_vel = torch.randn(NUM_ACTIONS)
    prev_actions = torch.randn(NUM_ACTIONS)
    imu_ang_vel = torch.randn(3)
    imu_euler_xyz = torch.randn(3)
    buffer = torch.zeros(574)
    input_tensors = (x_vel, y_vel, rot, t, dof_pos, dof_vel, prev_actions, imu_ang_vel, imu_euler_xyz, buffer)

    # Run the model once, for debugging.
    # a_model(*input_tensors)

    jit_model = torch.jit.script(a_model)
    torch.onnx.export(jit_model, input_tensors, "position_control.onnx")


if __name__ == "__main__":
    convert()
