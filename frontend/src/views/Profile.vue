<template>
  <div class="profile-container">
    <el-card class="profile-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <el-icon><User /></el-icon>
          <span>个人中心</span>
        </div>
      </template>

      <el-form
        ref="formRef"
        :model="formData"
        :rules="rules"
        label-width="100px"
        label-position="right"
        class="profile-form"
      >
        <el-form-item label="用户名">
          <el-input v-model="formData.username" disabled />
        </el-form-item>
        <el-form-item label="真实姓名" prop="real_name">
          <el-input v-model="formData.real_name" placeholder="请输入真实姓名" />
        </el-form-item>
        <el-form-item label="电话" prop="phone">
          <el-input v-model="formData.phone" placeholder="请输入电话号码" />
        </el-form-item>
        <el-form-item label="邮箱" prop="email">
          <el-input v-model="formData.email" placeholder="请输入邮箱地址" />
        </el-form-item>

        <el-divider>修改密码</el-divider>

        <el-form-item label="旧密码" prop="oldPassword">
          <el-input
            v-model="formData.oldPassword"
            type="password"
            placeholder="请输入旧密码"
            show-password
          />
        </el-form-item>
        <el-form-item label="新密码" prop="newPassword">
          <el-input
            v-model="formData.newPassword"
            type="password"
            placeholder="请输入新密码"
            show-password
            clearable
            @input="handleNewPasswordChange"
          />
        </el-form-item>
        <el-form-item label="确认密码" prop="confirmPassword">
          <el-input
            v-model="formData.confirmPassword"
            type="password"
            placeholder="请再次输入新密码"
            show-password
          />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" :loading="loading" @click="handleSubmit">
            保存修改
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from "vue";
import { ElMessage, type FormInstance, type FormRules } from "element-plus";
import { User } from "@element-plus/icons-vue";
import { useUserStore } from "@/stores/user";
import { updateUserSelf } from "@/api/user";
import type { UpdateUserParams } from "@/types/user";

const userStore = useUserStore();
const loading = ref(false);
const formRef = ref<FormInstance>();

const formData = reactive({
  username: "",
  real_name: "",
  phone: "",
  email: "",
  oldPassword: "",
  newPassword: "",
  confirmPassword: "",
});

const rules: FormRules = {
  real_name: [
    { max: 50, message: "真实姓名不能超过50个字符", trigger: "blur" },
  ],
  phone: [
    {
      pattern: /^1[3-9]\d{9}$/,
      message: "请输入正确的手机号",
      trigger: "blur",
    },
  ],
  email: [
    {
      type: "email",
      message: "请输入正确的邮箱地址",
      trigger: "blur",
    },
  ],
  oldPassword: [{ trigger: "blur" }],
  newPassword: [
    { min: 3, max: 20, message: "密码长度在3到20位之间", trigger: "blur" },
  ],
  confirmPassword: [
    {
      validator: (rule, value, callback) => {
        if (value !== formData.newPassword) {
          callback(new Error("两次输入的密码不一致"));
        } else {
          callback();
        }
      },
      trigger: "blur",
    },
  ],
};

// 当新密码变化时，重新校验确认密码
const handleNewPasswordChange = () => {
  if (formRef.value) {
    formRef.value.validateField("confirmPassword");
  }
};

// 加载当前用户信息
const loadUserInfo = () => {
  const user = userStore.userInfo;
  if (user) {
    formData.username = user.username || "";
    formData.real_name = user.real_name || "";
    formData.phone = user.phone || "";
    formData.email = user.email || "";
  }
};

// 提交修改
const handleSubmit = async () => {
  if (!formRef.value) return;
  await formRef.value.validate(async (valid) => {
    if (!valid) return;

    const params: UpdateUserParams = {
      id: userStore.userInfo!.id,
      real_name: formData.real_name,
      phone: formData.phone,
      email: formData.email,
    };

    // 如果填写了新密码，则添加旧密码和新密码
    if (formData.newPassword) {
      if (!formData.oldPassword) {
        ElMessage.warning("修改密码必须填写旧密码");
        return;
      }
      params.old_password = formData.oldPassword;
      params.password = formData.newPassword;
    }

    loading.value = true;
    try {
      await updateUserSelf(params);
      // 刷新用户信息（从后端重新获取）
      await userStore.fetchUserInfo();
      ElMessage.success("修改成功");
      // 清空密码字段
      formData.oldPassword = "";
      formData.newPassword = "";
      formData.confirmPassword = "";
      // 重新加载表单显示新数据
      loadUserInfo();
    } catch (error) {
      // 错误已在拦截器中处理
    } finally {
      loading.value = false;
    }
  });
};

onMounted(() => {
  loadUserInfo();
});
</script>

<style scoped>
.profile-container {
  max-width: 600px;
  margin: 0 auto;
  padding: 20px;
}
.profile-card {
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);
  box-shadow: 0 20px 35px -10px rgba(0, 0, 0, 0.1);
}
.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 500;
  color: #2c3e50;
}
.profile-form {
  margin-top: 20px;
}
</style>